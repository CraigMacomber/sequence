//! Simple tree that ChunkIds instead of its children.
//! Can be used with Forest to form a Tree.
//! Nav can combine this with a Forest to produce a Tree API with child access methods.

use std::{iter::Cloned, slice};

use crate::{
    forest::{self, ChunkId},
    util::{ImHashMap, ImSlice},
    Def, HasId, Label, Node, NodeId, NodeNav,
};

#[derive(Clone, PartialEq)]
pub struct BasicNode {
    pub def: Def,
    // Payload is often not used, so indirect it to keep the size down.
    pub payload: Option<Box<im_rc::Vector<u8>>>,
    // TODO: use im::Vector here
    pub traits: ImHashMap<Label, Vec<ChunkId>>,
}

impl<'a> NodeNav<ChunkId> for &'a BasicNode {
    type TTraitChildren = Cloned<slice::Iter<'a, ChunkId>>;
    type TLabels = Cloned<im_rc::hashmap::Keys<'a, Label, Vec<ChunkId>>>;

    fn get_traits(&self) -> Self::TLabels {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        self.traits
            .get(&label)
            .map_or(BasicNode::empty_trait(), |x| x.iter().cloned())
    }
}

impl Node<ChunkId> for &BasicNode {
    fn get_def(&self) -> Def {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        let o = &self.payload;
        match o {
            Some(p) => Some(p.focus()),
            None => None,
        }
    }
}

impl<'a> BasicNode {
    const EMPTY: [ChunkId; 0] = [];
    pub fn empty_trait() -> Cloned<slice::Iter<'a, ChunkId>> {
        Self::EMPTY.iter().cloned()
    }
}

/// View of a BasicNode with an Id.
#[derive(Clone)]
pub struct BasicView<'a> {
    pub node: &'a BasicNode,
    pub id: NodeId,
}

impl<'a> forest::Chunk for &'a BasicNode {
    type View = BasicView<'a>;
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<BasicView<'a>> {
        if first_id == id {
            Some(BasicView { node: self, id })
        } else {
            None
        }
    }
}

impl HasId for BasicView<'_> {
    fn get_id(&self) -> NodeId {
        self.id
    }
}

impl<'a> NodeNav<ChunkId> for BasicView<'a> {
    type TTraitChildren = Cloned<slice::Iter<'a, ChunkId>>;
    type TLabels = Cloned<im_rc::hashmap::Keys<'a, Label, Vec<ChunkId>>>;

    fn get_traits(&self) -> Self::TLabels {
        self.node.get_traits()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        self.node.get_trait(label)
    }
}

impl<'a> Node<ChunkId> for BasicView<'a> {
    fn get_def(&self) -> Def {
        self.node.get_def()
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.node.get_payload()
    }
}
