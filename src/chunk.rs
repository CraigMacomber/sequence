use std::iter::Cloned;

use crate::{
    util::{slice_with_length, ImSlice},
    Node, Trait,
};

pub struct Chunk {
    first_id: u128,
    schema: ChunkSchema,
    data: im_rc::Vector<u8>,
    id_offset_to_byte_offset_and_schema: im_rc::HashMap<u32, (u32, ChunkSchema)>, // TODO: include parent info in this
}

struct ChunkSchema {
    def: u128,
    node_count: u32,
    bytes_per_node: u32,
    id_stride: u32, // total number in subtree (nodes under traits + 1)
    payload_size: Option<u16>,
    traits: im_rc::HashMap<u128, OffsetSchema>,
}

pub struct OffsetSchema {
    id_offset: u32,
    byte_offset: u32,
    schema: ChunkSchema,
}

// Views

#[derive(Clone)]
pub struct ChunkView<'a> {
    first_id: u128,
    schema: &'a ChunkSchema,
    data: ImSlice<'a>,
}

impl<'a> Chunk {
    pub fn lookup(&'a self, id: u128) -> Option<ChunkOffset<'a>> {
        if id < self.first_id {
            None
        } else if id < self.first_id + (self.schema.id_stride as usize * self.get_count()) as u128 {
            let id_offset = (id - self.first_id) as u32;
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
    pub fn view(&self) -> ChunkView {
        ChunkView {
            first_id: self.first_id,
            schema: &self.schema,
            data: self.data.focus(),
        }
    }
}

impl<'a> ChunkOffset<'a> {
    fn first_id(&self) -> u128 {
        (self.offset * self.view.schema.id_stride) as u128 + self.view.first_id
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
    type TTrait = ChunkView<'a>;

    fn get_id(&self) -> u128 {
        self.view.first_id + self.offset as u128
    }

    fn get_def(&self) -> u128 {
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

    type TTraitIterator = Cloned<im_rc::hashmap::Keys<'a, u128, OffsetSchema>>;

    fn get_traits(&self) -> Self::TTraitIterator {
        self.view.schema.traits.keys().cloned()
    }

    fn get_trait(&self, label: u128) -> Option<Self::TTrait> {
        match self.view.schema.traits.get(&label) {
            Some(x) => Some(ChunkView {
                schema: &x.schema,
                data: slice_with_length(
                    self.data(),
                    x.byte_offset as usize,
                    x.schema.bytes_per_node as usize,
                ),
                first_id: self.first_id() + x.id_offset as u128,
            }),
            None => None,
        }
    }
}

// TODO: delete or use this
impl<'a> Iterator for ChunkOffset<'a> {
    type Item = ChunkOffset<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.view.schema.node_count {
            let out = self.clone();
            self.offset += 1;
            Some(out)
        } else {
            None
        }
    }
}

impl<'a> Trait<ChunkOffset<'a>> for ChunkView<'a> {
    fn get_count(&self) -> usize {
        self.schema.node_count as usize
    }
    fn get_child(&self, index: usize) -> ChunkOffset<'a> {
        let offset = self.schema.bytes_per_node as usize * index as usize;
        let view = ChunkView {
            first_id: self.first_id + (index * self.schema.id_stride as usize) as u128,
            schema: self.schema,
            data: slice_with_length(
                self.data.clone(),
                offset,
                self.schema.bytes_per_node as usize,
            ),
        };
        ChunkOffset::<'a> {
            view,
            offset: index as u32,
        }
    }
}
