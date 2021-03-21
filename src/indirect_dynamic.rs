use std::collections::HashMap;

use crate::{
    chunk::{Chunk, ChunkOffset, ChunkView},
    util::CloneIterator,
    Node, Trait,
};

pub struct DynamicNode {
    pub id: u128,
    pub def: u128,
    pub payload: Option<Vec<u8>>,
    pub traits: HashMap<u128, DynamicTrait>,
}

// DynamicNode | u128 id, or some other kind of id (reference, short id etc)
type ChildId = u128;

pub struct DynamicTrait {
    length: usize,
    children: Vec<Nodes>,
}

impl DynamicTrait {
    pub fn new(children: Vec<Nodes>) -> DynamicTrait {
        let mut length = 0;
        for n in &children {
            length += match n {
                Nodes::Dynamic(_) => 1,
                Nodes::Chunk(c) => c.get_count(),
            };
        }
        DynamicTrait { children, length }
    }
}

pub enum Nodes {
    Dynamic(ChildId),
    Chunk(Chunk),
}

enum NodeView<'a> {
    Dynamic(&'a DynamicNode),
    Chunk(ChunkOffset<'a>),
}

enum ChildView<'a> {
    Dynamic(ChildId),
    Chunk(ChunkOffset<'a>),
}

enum TraitView<'a> {
    Dynamic(&'a DynamicTrait),
    Chunk(ChunkView<'a>),
}

impl<'a> Node<ChildView<'a>> for NodeView<'a> {
    type TTrait = TraitView<'a>;
    type TTraitIterator = Vec<u128>;

    fn get_id(&self) -> u128 {
        match self {
            NodeView::Dynamic(d) => d.id,
            NodeView::Chunk(c) => c.get_id(),
        }
    }

    fn get_def(&self) -> u128 {
        match self {
            NodeView::Dynamic(d) => d.def,
            NodeView::Chunk(c) => c.get_def(),
        }
    }

    fn get_payload(&self) -> Option<&[u8]> {
        match self {
            NodeView::Dynamic(d) => match &d.payload {
                Some(p) => Some(p.as_slice()),
                None => None,
            },
            NodeView::Chunk(c) => c.get_payload(),
        }
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        match self {
            NodeView::Dynamic(d) => CloneIterator { t: d.traits.keys() }.collect(),
            NodeView::Chunk(c) => c.get_traits().collect(),
        }
    }

    fn get_trait(&self, label: u128) -> Option<Self::TTrait> {
        match self {
            NodeView::Dynamic(d) => d.traits.get(&label).map(|x| TraitView::Dynamic(x)),
            NodeView::Chunk(c) => c.get_trait(label).map(|x| TraitView::Chunk(x)),
        }
    }
}

impl<'a> Trait<ChildView<'a>> for TraitView<'a> {
    fn get_count(&self) -> usize {
        match self {
            TraitView::Dynamic(d) => d.length,
            TraitView::Chunk(c) => c.get_count(),
        }
    }
    // TODO: currently there is no trait iterator, and this is O(number of chunks). An iterator should be provided and/or this should be optimized.
    fn get_child(&self, index: usize) -> ChildView<'a> {
        match self {
            TraitView::Dynamic(d) => {
                let mut count: usize = 0;
                for n in &d.children {
                    match n {
                        Nodes::Dynamic(d) => {
                            if count == index {
                                return ChildView::Dynamic(*d);
                            }
                            count += 1;
                        }
                        Nodes::Chunk(c) => {
                            let more = c.get_count();
                            if count + more > index {
                                return ChildView::Chunk(ChunkOffset {
                                    view: c.view(),
                                    offset: (index - count) as u32,
                                });
                            }
                            count += more;
                        }
                    };
                }
                panic!("trait index out of range")
            }
            TraitView::Chunk(c) => ChildView::Chunk(c.get_child(index)),
        }
    }
}
