use std::collections::{hash_map::Keys, HashMap};

use crate::{util::CloneIterator, Node, Trait};

pub struct Chunk {
    first_id: u128,
    schema: ChunkSchema,
    data: Vec<u8>,
}

struct ChunkSchema {
    def: u128,
    node_count: u32,
    bytes_per_node: u16,
    id_stride: u32, // Typically total number in subtree (nodes under traits + 1)
    payload_size: Option<u16>,
    traits: HashMap<u128, OffsetSchema>,
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
    data: &'a [u8],
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
            data: &self.data,
        }
    }
}

impl<'a> ChunkOffset<'a> {
    fn first_id(&self) -> u128 {
        (self.offset * self.view.schema.id_stride) as u128 + self.view.first_id
    }

    fn data(&self) -> &'a [u8] {
        let offset = self.offset as usize;
        let stride = self.view.schema.bytes_per_node as usize;
        let start = offset * stride;
        &self.view.data[start..start + stride]
    }
}

// Views first item as chunk in as node
impl<'a> Node for ChunkOffset<'a> {
    type TTrait = ChunkView<'a>;

    fn get_id(&self) -> u128 {
        self.view.first_id + self.offset as u128
    }

    fn get_def(&self) -> u128 {
        self.view.schema.def
    }

    fn get_payload(&self) -> Option<&[u8]> {
        let offset = self.offset as usize * self.view.schema.bytes_per_node as usize;
        match self.view.schema.payload_size {
            Some(p) => Some(&self.view.data[offset..offset + p as usize]),
            None => None,
        }
    }

    type TTraitIterator = CloneIterator<Keys<'a, u128, OffsetSchema>>;

    fn get_traits(&self) -> Self::TTraitIterator {
        CloneIterator {
            t: self.view.schema.traits.keys(),
        }
    }

    fn get_trait(&self, label: u128) -> Option<Self::TTrait> {
        match self.view.schema.traits.get(&label) {
            Some(x) => Some(ChunkView {
                schema: &x.schema,
                data: &self.data()[x.byte_offset as usize
                    ..x.byte_offset as usize + x.schema.bytes_per_node as usize],
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

impl<'a> Trait for ChunkView<'a> {
    type TNode = ChunkOffset<'a>;

    fn get_count(&self) -> usize {
        self.schema.node_count as usize
    }
    fn get_child(&self, index: usize) -> Self::TNode {
        let offset = self.schema.bytes_per_node as usize * index as usize;
        let view = ChunkView {
            first_id: self.first_id + (index * self.schema.id_stride as usize) as u128,
            schema: self.schema,
            data: &self.data[offset..offset + self.schema.bytes_per_node as usize],
        };
        ChunkOffset::<'a> {
            view,
            offset: index as u32,
        }
    }
}
