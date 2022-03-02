//! Simple tree that has ChunkIds as its children.
//! Can be used with Forest to form a Tree.
//! Nav can combine this with a Forest to produce a Tree API with child access methods.

use std::{iter::Cloned, slice};

use crate::{
    chunk::Chunk,
    forest::ChunkId,
    util::{ImHashMap, ImSlice},
    Def, HasId, Label, NodeData, NodeId, NodeNav,
};

#[derive(Clone, PartialEq)]
pub struct IndirectNode {
    pub def: Def,
    // Payload is often not used, so indirect it to keep the size down.
    pub payload: Option<Box<im_rc::Vector<u8>>>,
    // TODO: use im::Vector here
    pub traits: ImHashMap<Label, Vec<ChunkId>>,
}

impl<'a> NodeNav<ChunkId> for &'a IndirectNode {
    type TTraitChildren = Cloned<slice::Iter<'a, ChunkId>>;
    type TLabels = Cloned<im_rc::hashmap::Keys<'a, Label, Vec<ChunkId>>>;

    fn get_traits(&self) -> Self::TLabels {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        self.traits
            .get(&label)
            .map_or(IndirectNode::empty_trait(), |x| x.iter().cloned())
    }
}

impl NodeData for IndirectNode {
    fn get_def(&self) -> Def {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.payload.as_ref().map(|p| p.focus())
    }
}

impl IndirectNode {
    const EMPTY: [ChunkId; 0] = [];
    pub fn empty_trait() -> Cloned<slice::Iter<'static, ChunkId>> {
        Self::EMPTY.iter().cloned()
    }
}

/// View of a BasicNode with an Id.
#[derive(Clone)]
pub struct BasicView<'a> {
    pub node: &'a IndirectNode,
    pub id: NodeId,
}

impl<'a> Chunk for &'a IndirectNode {
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

impl NodeData for BasicView<'_> {
    fn get_def(&self) -> Def {
        self.node.get_def()
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.node.get_payload()
    }
}
