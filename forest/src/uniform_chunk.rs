//! Sequence of trees with identical schema and sequential ids (depth first pre-order).
//! Owns the content. Compressed (one copy of schema, rest as blob)

use std::{iter::Cloned, rc::Rc, usize};

use crate::{
    chunk::Chunk,
    util::{slice_with_length, ImSlice},
    Def, HasId, IdOffset, Label, Node, NodeId, NodeNav,
};

#[derive(Clone)]
pub struct UniformChunk {
    pub data: Box<im_rc::Vector<u8>>,
    pub schema: Rc<RootChunkSchema>,
}

impl PartialEq for UniformChunk {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.schema, &other.schema) & self.data.eq(&other.data)
    }
}

pub struct RootChunkSchema {
    pub schema: ChunkSchema,
    // TODO: include parent info in this
    /// Derived data (from schema) to enable fast lookup of views from id.
    id_offset_to_byte_offset_and_schema: Vec<Option<OffsetInfo>>,
}

#[derive(Clone)]
struct OffsetInfo {
    byte_offset: u32,
    schema: ChunkSchema,
    parent: ParentInfo,
}

#[derive(Clone)]
pub struct ParentInfo {
    /// None for top level nodes in chunk
    pub parent: Option<(IdOffset, Label)>,
    pub index: usize,
}

#[derive(Clone)]
pub struct OffsetInfoRef<'a> {
    pub byte_offset: u32,
    pub schema: &'a ChunkSchema,
    pub parent: ParentInfo,
}

impl RootChunkSchema {
    pub fn new(schema: ChunkSchema) -> Self {
        let mut data_outer = vec![None; schema.id_stride as usize];

        fn add(
            data: &mut [Option<OffsetInfo>],
            s: &ChunkSchema,
            byte_offset: u32,
            id_offset: usize,
            parent: ParentInfo,
        ) {
            debug_assert!(data[id_offset].is_none());
            data[id_offset] = Some(OffsetInfo {
                byte_offset,
                schema: s.clone(),
                parent,
            });
            for (label, sub_schema) in s.traits.iter() {
                for i in 0..sub_schema.schema.node_count {
                    add(
                        data,
                        &sub_schema.schema,
                        byte_offset + sub_schema.byte_offset + i * sub_schema.schema.bytes_per_node,
                        id_offset
                            + sub_schema.id_offset.0 as usize
                            + i as usize * sub_schema.schema.id_stride as usize,
                        ParentInfo {
                            parent: Some((IdOffset(id_offset as u32), *label)),
                            index: i as usize,
                        },
                    )
                }
            }
        }

        add(
            &mut data_outer.as_mut_slice(),
            &schema,
            0,
            0,
            ParentInfo {
                parent: None,
                index: 0,
            },
        );

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

impl<'a> Chunk for &'a UniformChunk {
    type View = ChunkOffset<'a>;
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<ChunkOffset<'a>> {
        match self.schema.lookup_schema(first_id, id) {
            Some(info) => {
                let data = slice_with_length(
                    self.data.focus(),
                    info.byte_offset as usize,
                    info.schema.bytes_per_node as usize,
                );
                let view = ChunkView {
                    first_id: id,
                    schema: &info.schema,
                    data,
                };
                Some(ChunkOffset { view, offset: 0 })
            }
            None => None,
        }
    }
}

impl RootChunkSchema {
    /// Returns None if id not present.
    pub fn lookup_schema(&self, first_id: NodeId, id: NodeId) -> Option<OffsetInfoRef> {
        if id < first_id {
            None
        } else if id < first_id + IdOffset(self.schema.id_stride * self.schema.node_count) {
            let id_offset = (id - first_id).0;
            let (div, rem) = num_integer::div_rem(id_offset, self.schema.id_stride);
            let info = self.id_offset_to_byte_offset_and_schema[rem as usize].as_ref()?;
            let byte_offset = info.byte_offset + div * self.schema.bytes_per_node;
            let parent = ParentInfo {
                parent: info.parent.parent,
                index: match info.parent.parent {
                    Some(_) => info.parent.index, // Index of parent? Used as index of child?
                    None => div as usize, // This is index within chunk at chunk top level, not index within trait.
                },
            };
            Some(OffsetInfoRef {
                byte_offset,
                schema: &info.schema,
                parent,
            })
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

impl UniformChunk {
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
    fn data(&self) -> ImSlice<'a> {
        let offset = self.offset as usize;
        let stride = self.view.schema.bytes_per_node as usize;
        let start = offset * stride;
        slice_with_length(self.view.data.clone(), start, stride)
    }
}

impl<'a> NodeNav<ChunkOffset<'a>> for ChunkOffset<'a> {
    type TTraitChildren = ChunkIterator<'a>;
    type TLabels = Cloned<std::collections::hash_map::Keys<'a, Label, OffsetSchema>>;

    fn get_traits(&self) -> Self::TLabels {
        self.view.schema.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        match self.view.schema.traits.get(&label) {
            Some(x) => {
                let node_data = self.data();
                let trait_data = slice_with_length(
                    node_data,
                    x.byte_offset as usize,
                    x.schema.bytes_per_node as usize,
                );
                let trait_first_id = self.get_id() + x.id_offset;
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
impl<'a> Node for ChunkOffset<'a> {
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
}

// Views first item as chunk in as node
impl HasId for ChunkOffset<'_> {
    fn get_id(&self) -> NodeId {
        self.view.first_id + IdOffset(self.offset * self.view.schema.id_stride)
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
