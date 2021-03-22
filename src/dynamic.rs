use std::collections::HashMap;

use crate::{
    chunk::{Chunk, ChunkOffset, ChunkView},
    util::ImSlice,
    Node, Trait,
};

pub struct DynamicNode {
    pub id: u128,
    pub def: u128,
    pub payload: Option<im_rc::Vector<u8>>,
    pub traits: HashMap<u128, DynamicTrait>,
}

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
    Dynamic(DynamicNode),
    Chunk(Chunk),
}

enum NodesView<'a> {
    Dynamic(&'a DynamicNode),
    Chunk(ChunkOffset<'a>),
}

enum TraitView<'a> {
    Dynamic(&'a DynamicTrait),
    Chunk(ChunkView<'a>),
}

impl<'a> Node<NodesView<'a>> for NodesView<'a> {
    type TTrait = TraitView<'a>;
    type TTraitIterator = Vec<u128>;

    fn get_id(&self) -> u128 {
        match self {
            NodesView::Dynamic(d) => d.id,
            NodesView::Chunk(c) => c.get_id(),
        }
    }

    fn get_def(&self) -> u128 {
        match self {
            NodesView::Dynamic(d) => d.def,
            NodesView::Chunk(c) => c.get_def(),
        }
    }

    fn get_payload(&self) -> Option<ImSlice> {
        match self {
            NodesView::Dynamic(d) => d.payload.as_ref().map(|p| p.focus()),
            NodesView::Chunk(c) => c.get_payload(),
        }
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        match self {
            NodesView::Dynamic(d) => d.traits.keys().cloned().collect(),
            NodesView::Chunk(c) => c.get_traits().collect(),
        }
    }

    fn get_trait(&self, label: u128) -> Option<Self::TTrait> {
        match self {
            NodesView::Dynamic(d) => d.traits.get(&label).map(|x| TraitView::Dynamic(x)),
            NodesView::Chunk(c) => c.get_trait(label).map(|x| TraitView::Chunk(x)),
        }
    }
}

impl<'a> Trait<NodesView<'a>> for TraitView<'a> {
    fn get_count(&self) -> usize {
        match self {
            TraitView::Dynamic(d) => d.length,
            TraitView::Chunk(c) => c.get_count(),
        }
    }
    // TODO: currently there is no trait iterator, and this is O(number of chunks). An iterator should be provided and/or this should be optimized.
    fn get_child(&self, index: usize) -> NodesView<'a> {
        match self {
            TraitView::Dynamic(d) => {
                let mut count: usize = 0;
                for n in &d.children {
                    match n {
                        Nodes::Dynamic(d) => {
                            if count == index {
                                return NodesView::Dynamic(d);
                            }
                            count += 1;
                        }
                        Nodes::Chunk(c) => {
                            let more = c.get_count();
                            if count + more > index {
                                return NodesView::Chunk(ChunkOffset {
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
            TraitView::Chunk(c) => NodesView::Chunk(c.get_child(index)),
        }
    }
}
