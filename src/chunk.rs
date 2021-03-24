//! Sequence of trees with identical schema and sequential ids (depth first pre-order).
//! Owns the content. Compressed (one copy of schema, rest as blob)

use std::iter::Cloned;

use crate::{
    util::{slice_with_length, ImSlice},
    Def, HasId, IdOffset, Label, Node, NodeId,
};

#[derive(Clone)]
pub struct Chunk {
    schema: ChunkSchema,
    data: im_rc::Vector<u8>,
    id_offset_to_byte_offset_and_schema: im_rc::HashMap<u32, (u32, ChunkSchema)>, // TODO: include parent info in this
}

#[derive(Clone)]
struct ChunkSchema {
    def: Def,
    /// total number in subtree (nodes under traits + 1)
    node_count: u32,
    bytes_per_node: u32,
    id_stride: u32,
    payload_size: Option<u16>,
    traits: im_rc::HashMap<Label, OffsetSchema>,
}

#[derive(Clone)]
pub struct OffsetSchema {
    id_offset: IdOffset,
    byte_offset: u32,
    schema: ChunkSchema,
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
        } else if id < first_id + IdOffset(self.schema.id_stride * self.get_count() as u32) {
            let id_offset = (id - first_id).0;
            let (div, rem) = num_integer::div_rem(id_offset, self.schema.id_stride);
            let (inner_byte_offset, schema) =
                self.id_offset_to_byte_offset_and_schema.get(&rem).unwrap();
            let byte_offset = inner_byte_offset + div * self.schema.bytes_per_node;
            let data = slice_with_length(
                self.data.focus(),
                byte_offset as usize,
                schema.bytes_per_node as usize,
            );
            let view = ChunkView {
                first_id: id,
                schema,
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
        self.schema.node_count as usize
    }
    pub fn view(&self, id: NodeId) -> ChunkView {
        ChunkView {
            first_id: id,
            schema: &self.schema,
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
        let offset = self.offset as usize * self.view.schema.bytes_per_node as usize;
        match self.view.schema.payload_size {
            Some(p) => Some(slice_with_length(
                self.view.data.clone(),
                offset,
                p as usize,
            )),
            None => None,
        }
    }

    type TTraitIterator = Cloned<im_rc::hashmap::Keys<'a, Label, OffsetSchema>>;

    fn get_traits(&self) -> Self::TTraitIterator {
        self.view.schema.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        match self.view.schema.traits.get(&label) {
            Some(x) => ChunkIterator::View(ChunkOffset {
                offset: 0,
                view: ChunkView {
                    schema: &x.schema,
                    data: slice_with_length(
                        self.data(),
                        x.byte_offset as usize,
                        x.schema.bytes_per_node as usize,
                    ),
                    first_id: self.first_id() + x.id_offset,
                },
            }),
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
