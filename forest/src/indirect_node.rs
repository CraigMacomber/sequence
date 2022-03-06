//! Simple tree that has ChunkIds as its children.
//! Can be used with Forest to form a Tree.
//! Nav can combine this with a Forest to produce a Tree API with child access methods.

use std::{iter::Cloned, slice};

use crate::{
    chunk::Chunk,
    forest::ChunkId,
    node_id::{HasId, NodeId},
    tree::{Def, Label, NodeData, NodeNav},
    util::{ImHashMap, ImSlice},
};

#[derive(Clone, PartialEq)]
pub struct IndirectChunk {
    pub def: Def,
    // Payload is often not used, so indirect it to keep the size down.
    pub payload: Option<Box<im_rc::Vector<u8>>>,
    // TODO: use im::Vector here
    pub traits: ImHashMap<Label, Vec<ChunkId>>,
}

impl<'a> NodeNav<ChunkId> for &'a IndirectChunk {
    type TTraitChildren = Cloned<slice::Iter<'a, ChunkId>>;
    type TLabels = Cloned<im_rc::hashmap::Keys<'a, Label, Vec<ChunkId>>>;

    fn get_traits(&self) -> Self::TLabels {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        self.traits
            .get(&label)
            .map_or(IndirectChunk::empty_trait(), |x| x.iter().cloned())
    }
}

impl NodeData for IndirectChunk {
    fn get_def(&self) -> Def {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.payload.as_ref().map(|p| p.focus())
    }
}

impl IndirectChunk {
    const EMPTY: [ChunkId; 0] = [];
    pub fn empty_trait() -> Cloned<slice::Iter<'static, ChunkId>> {
        Self::EMPTY.iter().cloned()
    }
}

/// View of a BasicNode with an Id.
#[derive(Clone)]
pub struct IndirectNode<'a> {
    pub node: &'a IndirectChunk,
    pub id: NodeId,
}

impl<'a> Chunk for &'a IndirectChunk {
    type View = IndirectNode<'a>;
    type Child = ChunkId;
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<IndirectNode<'a>> {
        if first_id == id {
            Some(IndirectNode { node: self, id })
        } else {
            None
        }
    }
}

impl HasId for IndirectNode<'_> {
    fn get_id(&self) -> NodeId {
        self.id
    }
}

impl<'a> NodeNav<ChunkId> for IndirectNode<'a> {
    type TTraitChildren = Cloned<slice::Iter<'a, ChunkId>>;
    type TLabels = Cloned<im_rc::hashmap::Keys<'a, Label, Vec<ChunkId>>>;

    fn get_traits(&self) -> Self::TLabels {
        self.node.get_traits()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        self.node.get_trait(label)
    }
}

impl NodeData for IndirectNode<'_> {
    fn get_def(&self) -> Def {
        self.node.get_def()
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.node.get_payload()
    }
}
