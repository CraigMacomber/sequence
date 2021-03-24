//! Sequence of trees with identical schema and sequential ids (depth first pre-order).
//! Owns the content. Compressed (one copy of schema, rest as blob)

use std::{
    iter::Cloned,
    ops::{Add, Sub},
};

use crate::{
    util::{slice_with_length, ImSlice},
    Def, IdOffset, Label, Node,
};

#[derive(Clone)]
pub struct Chunk<Id> {
    first_id: Id,
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
pub struct ChunkView<'a, Id> {
    first_id: Id,
    schema: &'a ChunkSchema,
    data: ImSlice<'a>,
}

pub trait IdConstraint<Id>:
    Copy + Ord + Sub<Id, Output = IdOffset> + Add<IdOffset, Output = Id>
{
}

impl<Id: Copy + Clone + Ord + Sub<Id, Output = IdOffset> + Add<IdOffset, Output = Id>>
    IdConstraint<Id> for Id
{
}

impl<'a, Id> Chunk<Id>
where
    Id: IdConstraint<Id>,
{
    pub fn lookup(&'a self, id: Id) -> Option<ChunkOffset<'a, Id>> {
        if id < self.first_id {
            None
        } else if id < self.first_id + IdOffset(self.schema.id_stride * self.get_count() as u32) {
            let id_offset = (id - self.first_id).0;
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
pub struct ChunkOffset<'a, Id> {
    pub view: ChunkView<'a, Id>,
    pub offset: u32, // index of current node in ChunkView
}

impl<Id: Copy> Chunk<Id> {
    pub fn get_count(&self) -> usize {
        self.schema.node_count as usize
    }
    pub fn view(&self) -> ChunkView<Id> {
        ChunkView {
            first_id: self.first_id,
            schema: &self.schema,
            data: self.data.focus(),
        }
    }
}

impl<'a, Id: IdConstraint<Id>> ChunkOffset<'a, Id> {
    fn first_id(&self) -> Id {
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
impl<'a, Id: IdConstraint<Id>> Node<ChunkOffset<'a, Id>, Id> for ChunkOffset<'a, Id>
where
    Id: std::ops::Add<IdOffset>,
{
    type TTrait = ChunkIterator<'a, Id>;

    fn get_id(&self) -> Id {
        self.view.first_id + IdOffset(self.offset)
    }

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

pub enum ChunkIterator<'a, T> {
    View(ChunkOffset<'a, T>),
    Empty,
}

impl<'a, Id: Clone> Iterator for ChunkIterator<'a, Id> {
    type Item = ChunkOffset<'a, Id>;

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
