//! Sequence of trees with identical schema and sequential ids (depth first pre-order).
//! Owns the content. Compressed (one copy of schema, rest as blob)

use std::{iter::Cloned, rc::Rc};

use crate::{
    util::{slice_with_length, ImSlice},
    Def, HasId, IdOffset, Label, Node, NodeId,
};

#[derive(Clone)]
pub struct Chunk {
    pub data: Box<im_rc::Vector<u8>>,
    pub schema: Rc<RootChunkSchema>,
}

pub struct RootChunkSchema {
    pub schema: ChunkSchema,
    // TODO: include parent info in this
    /// Derived data (from schema) to enable fast lookup of views from id.
    id_offset_to_byte_offset_and_schema: Vec<Option<(u32, ChunkSchema)>>,
}

impl RootChunkSchema {
    pub fn new(schema: ChunkSchema) -> Self {
        let mut data_outer = vec![None; schema.id_stride as usize];

        fn add(
            data: &mut [Option<(u32, ChunkSchema)>],
            s: &ChunkSchema,
            byte_offset: u32,
            id_offset: usize,
        ) {
            debug_assert!(data[id_offset].is_none());
            data[id_offset] = Some((byte_offset, s.clone()));
            for sub_schema in s.traits.values() {
                for i in 0..sub_schema.schema.node_count {
                    add(
                        data,
                        &sub_schema.schema,
                        byte_offset + sub_schema.byte_offset + i * sub_schema.schema.bytes_per_node,
                        id_offset
                            + sub_schema.id_offset.0 as usize
                            + i as usize * sub_schema.schema.id_stride as usize,
                    )
                }
            }
        };

        add(&mut data_outer.as_mut_slice(), &schema, 0, 0);

        RootChunkSchema {
            schema,
            id_offset_to_byte_offset_and_schema: data_outer,
        }
    }
}

#[derive(Clone)]
pub struct ChunkSchema {
    pub def: Def,
    /// number of nodes at this level
    pub node_count: u32,
    pub bytes_per_node: u32,
    /// total number in subtree (nodes under traits + 1)
    pub id_stride: u32,
    pub payload_size: Option<u16>,
    pub traits: std::collections::HashMap<Label, OffsetSchema, ahash::RandomState>,
}

/// Offsets are for the first iteration (of a possible schema.node_count iterations)
/// and are relative to the immediate parent (the node not the trait).
/// Thus these offsets need to account for the parent's payload, the parent's id,
/// and all traits which precede this one (including their repetitions via node_count).
/// Note thats its allowed the layout in id space and byte space to differ, so which traits are preceding in each might not be the same.
/// Its also allowed to leave unused gaps in either id space or byte space.
#[derive(Clone)]
pub struct OffsetSchema {
    pub id_offset: IdOffset,
    pub byte_offset: u32,
    pub schema: ChunkSchema,
}

// Views

#[derive(Clone)]
pub struct ChunkView<'a> {
    first_id: NodeId,
    schema: &'a ChunkSchema,
    data: ImSlice<'a>,
}

impl<'a> Chunk {
    pub fn lookup(&'a self, first_id: NodeId, id: NodeId) -> Option<ChunkOffset<'a>> {
        if id < first_id {
            None
        } else if id < first_id + IdOffset(self.schema.schema.id_stride * self.get_count() as u32) {
            let id_offset = (id - first_id).0;
            let (div, rem) = num_integer::div_rem(id_offset, self.schema.schema.id_stride);
            let (inner_byte_offset, schema) = &self.schema.id_offset_to_byte_offset_and_schema
                [rem as usize]
                .as_ref()
                .unwrap();
            let byte_offset = inner_byte_offset + div * self.schema.schema.bytes_per_node;
            let data = slice_with_length(
                self.data.focus(),
                byte_offset as usize,
                schema.bytes_per_node as usize,
            );
            let view = ChunkView {
                first_id: id,
                schema: &schema,
                data,
            };
            Some(ChunkOffset { view, offset: 0 })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct ChunkOffset<'a> {
    pub view: ChunkView<'a>,
    pub offset: u32, // index of current node in ChunkView
}

impl Chunk {
    pub fn get_count(&self) -> usize {
        self.schema.schema.node_count as usize
    }
    pub fn view(&self, id: NodeId) -> ChunkView {
        ChunkView {
            first_id: id,
            schema: &self.schema.schema,
            data: self.data.focus(),
        }
    }
}

impl<'a> ChunkOffset<'a> {
    fn first_id(&self) -> NodeId {
        self.view.first_id + IdOffset(self.offset as u32 * self.view.schema.id_stride as u32)
    }

    fn data(&self) -> ImSlice<'a> {
        let offset = self.offset as usize;
        let stride = self.view.schema.bytes_per_node as usize;
        let start = offset * stride;
        slice_with_length(self.view.data.clone(), start, stride)
    }
}

// Views first item as chunk in as node
impl<'a> Node<ChunkOffset<'a>> for ChunkOffset<'a> {
    type TTrait = ChunkIterator<'a>;

    fn get_def(&self) -> Def {
        self.view.schema.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        match self.view.schema.payload_size {
            Some(p) => {
                let node_data = self.data();
                Some(slice_with_length(node_data, 0, p as usize))
            }
            None => None,
        }
    }

    type TTraitIterator = Cloned<std::collections::hash_map::Keys<'a, Label, OffsetSchema>>;

    fn get_traits(&self) -> Self::TTraitIterator {
        self.view.schema.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        match self.view.schema.traits.get(&label) {
            Some(x) => {
                let node_data = self.data();
                let trait_data = slice_with_length(
                    node_data,
                    x.byte_offset as usize,
                    x.schema.bytes_per_node as usize,
                );
                let trait_first_id = self.first_id() + x.id_offset;
                ChunkIterator::View(ChunkOffset {
                    offset: 0,
                    view: ChunkView {
                        schema: &x.schema,
                        data: trait_data,
                        first_id: trait_first_id,
                    },
                })
            }
            None => ChunkIterator::Empty,
        }
    }
}

// Views first item as chunk in as node
impl<'a> HasId for ChunkOffset<'a> {
    fn get_id(&self) -> NodeId {
        self.view.first_id + IdOffset(self.offset)
    }
}

pub enum ChunkIterator<'a> {
    View(ChunkOffset<'a>),
    Empty,
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = ChunkOffset<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ChunkIterator::View(ref mut offset) => {
                if offset.offset < offset.view.schema.node_count {
                    let out = offset.clone();
                    offset.offset += 1;
                    Some(out)
                } else {
                    None
                }
            }
            ChunkIterator::Empty => None,
        }
    }
}
