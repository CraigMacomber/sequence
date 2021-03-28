//! Utilities for viewing trees of mixed [BasicNode] and [crate::chunk::Chunk].
//! where the children of the basic nodes are [ChunkId]s which correspond to either a chunk, or a BasicNode.
//!
//! The [Node] implemented here is abstracts this as children are which are either Nodes, or ChunkIds.
//! When traversing within a chunk, no indirection is involved, looking up an Id is only required when traversing a BasicNode.
//!
//! Abstracting away this difference
//! (and dealing with the fact that a trait may contain a mix of chunks and basic nodes, and the chunks might contain multiple top level nodes)
//! is done by [crate::indirect_nav] which wraps this node in a Node implementation up with a forest using [crate::nav::Nav].

use std::rc::Rc;

use crate::{
    basic_indirect::BasicView, chunk::ChunkOffset, forest::ChunkId, Def, HasId, Label, Node,
    NodeId, NodeNav,
};

/// Child type for the [Node].
pub enum Child<'a> {
    /// Parent is a [BasicNode]: may resolve to either a [BasicNode] or a [crate::chunk::Chunk]
    Id(ChunkId),
    /// A node within a [crate::chunk::Chunk]: can the the child of another node within the same chunk.
    Chunk(ChunkOffset<'a>),
}

pub enum TraitView<'a> {
    Basic(<&'a BasicView<'a> as NodeNav<ChunkId>>::TTraitChildren),
    Chunk(<ChunkOffset<'a> as NodeNav<ChunkOffset<'a>>>::TTraitChildren),
    Dyn(Box<dyn Iterator<Item = Child<'a>> + 'a>),
}

pub trait DynView<'a>: Node<Child<'a>> + HasId {}

#[derive(Clone)]
pub enum NodeView<'a> {
    Single(BasicView<'a>),
    Chunk(ChunkOffset<'a>),
    // TODO: support undownloaded chunks blobs (find can return which blobs and at what offset the node is at)
    // TODO: support undownloaded subtrees that arn't chunks: find returns iterator of candidate trees using bloom filters
    // TODO: these types are write optimized. Consider supporting read/size optimized types (ex: using byte array instead of im's Vector)
    // TODO: maybe chunks referencing external subtrees (so they can have child references like payloads)
    Dyn(DynViewFull<'a>),
}

pub type DynViewFull<'a> =
    Rc<dyn DynView<'a, TTraitChildren = TraitView<'a>, TLabels = LabelIterator<'a>> + 'a>;

impl<'a> NodeNav<Child<'a>> for NodeView<'a> {
    type TTraitChildren = TraitView<'a>;

    type TLabels = LabelIterator<'a>;

    fn get_traits(&self) -> Self::TLabels {
        match self {
            NodeView::Single(s) => LabelIterator::Single(s.node.get_traits()),
            NodeView::Chunk(c) => LabelIterator::Chunk(c.get_traits()),
            NodeView::Dyn(s) => s.get_traits(),
        }
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        match self {
            NodeView::Single(s) => TraitView::Basic(s.node.get_trait(label)),
            NodeView::Chunk(c) => TraitView::Chunk(c.get_trait(label)),
            NodeView::Dyn(s) => s.get_trait(label),
        }
    }
}

impl<'a> Node<Child<'a>> for NodeView<'a> {
    fn get_def(&self) -> Def {
        match self {
            NodeView::Single(s) => s.node.get_def(),
            NodeView::Chunk(c) => c.get_def(),
            NodeView::Dyn(s) => s.get_def(),
        }
    }

    fn get_payload(&self) -> Option<crate::util::ImSlice> {
        match self {
            NodeView::Single(s) => s.node.get_payload(),
            NodeView::Chunk(c) => c.get_payload(),
            NodeView::Dyn(s) => s.get_payload(),
        }
    }
}

impl HasId for NodeView<'_> {
    fn get_id(&self) -> NodeId {
        match self {
            NodeView::Single(s) => s.id,
            NodeView::Chunk(c) => c.get_id(),
            NodeView::Dyn(s) => s.get_id(),
        }
    }
}

impl<'a> Iterator for TraitView<'a> {
    type Item = Child<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TraitView::Basic(ref mut c) => c.next().map(|id| Child::Id(id)),
            TraitView::Chunk(ref mut c) => c.next().map(|c| Child::Chunk(c)),
            TraitView::Dyn(ref mut c) => c.next(),
        }
    }
}

pub enum LabelIterator<'a> {
    Single(<&'a BasicView<'a> as NodeNav<ChunkId>>::TLabels),
    Chunk(<ChunkOffset<'a> as NodeNav<ChunkOffset<'a>>>::TLabels),
    Dyn(Box<dyn Iterator<Item = Label> + 'a>),
}

impl Iterator for LabelIterator<'_> {
    type Item = Label;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LabelIterator::Single(ref mut s) => s.next(),
            LabelIterator::Chunk(ref mut c) => c.next(),
            LabelIterator::Dyn(ref mut c) => c.next(),
        }
    }
}
