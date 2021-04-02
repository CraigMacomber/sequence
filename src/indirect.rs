//! Utilities for viewing trees of mixed [BasicNode] and [crate::chunk::Chunk].
//! where the children of the basic nodes are [ChunkId]s which correspond to either a chunk, or a BasicNode.
//!
//! The [Node] implemented here is abstracts this as children are which are either Nodes, or ChunkIds.
//! When traversing within a chunk, no indirection is involved, looking up an Id is only required when traversing a BasicNode.
//!
//! Abstracting away this difference
//! (and dealing with the fact that a trait may contain a mix of chunks and basic nodes, and the chunks might contain multiple top level nodes)
//! is done by [crate::indirect_nav] which wraps this node in a Node implementation up with a forest using [crate::nav::Nav].

use enum_dispatch::enum_dispatch;

use crate::{basic_indirect::BasicView, chunk::ChunkOffset, forest::ChunkId, Label, NodeNav};

/// Child type for the [Node].
pub enum Child<'a> {
    /// Parent is a [BasicNode]: may resolve to either a [BasicNode] or a [crate::chunk::Chunk]
    Id(ChunkId),
    /// A node within a [crate::chunk::Chunk]: can the the child of another node within the same chunk.
    Chunk(ChunkOffset<'a>),
}

pub enum TraitView<'a> {
    Basic(<BasicView<'a> as NodeNav<ChunkId>>::TTraitChildren),
    Chunk(<ChunkOffset<'a> as NodeNav<ChunkOffset<'a>>>::TTraitChildren),
}

#[enum_dispatch]
pub trait DynView<'a> {
    fn get_traits(&self) -> LabelIterator<'a>;
    fn get_trait(&self, label: Label) -> TraitView<'a>;
}

impl<'a> DynView<'a> for BasicView<'a> {
    fn get_traits(&self) -> LabelIterator<'a> {
        LabelIterator::Single(<Self as NodeNav<ChunkId>>::get_traits(self))
    }

    fn get_trait(&self, label: Label) -> TraitView<'a> {
        TraitView::Basic(<Self as NodeNav<ChunkId>>::get_trait(self, label))
    }
}

impl<'a> DynView<'a> for ChunkOffset<'a> {
    fn get_traits(&self) -> LabelIterator<'a> {
        LabelIterator::Chunk(<Self as NodeNav<ChunkOffset<'a>>>::get_traits(self))
    }

    fn get_trait(&self, label: Label) -> TraitView<'a> {
        TraitView::Chunk(<Self as NodeNav<ChunkOffset<'a>>>::get_trait(self, label))
    }
}

#[enum_dispatch(HasId, DynView, Node)] // NodeNav
#[derive(Clone)]
pub enum NodeView<'a> {
    Single(BasicView<'a>),
    Chunk(ChunkOffset<'a>),
    // TODO: support undownloaded chunks blobs (find can return which blobs and at what offset the node is at)
    // TODO: support undownloaded subtrees that arn't chunks: find returns iterator of candidate trees using bloom filters
    // TODO: these types are write optimized. Consider supporting read/size optimized types (ex: using byte array instead of im's Vector)
    // TODO: maybe chunks referencing external subtrees (so they can have child references like payloads)
}

impl<'a> NodeNav<Child<'a>> for NodeView<'a> {
    type TTraitChildren = TraitView<'a>;

    type TLabels = LabelIterator<'a>;

    fn get_traits(&self) -> Self::TLabels {
        <Self as DynView>::get_traits(&self)
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        <Self as DynView>::get_trait(&self, label)
    }
}

impl<'a> Iterator for TraitView<'a> {
    type Item = Child<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TraitView::Basic(ref mut c) => c.next().map(|id| Child::Id(id)),
            TraitView::Chunk(ref mut c) => c.next().map(|c| Child::Chunk(c)),
        }
    }
}

pub enum LabelIterator<'a> {
    Single(<BasicView<'a> as NodeNav<ChunkId>>::TLabels),
    Chunk(<ChunkOffset<'a> as NodeNav<ChunkOffset<'a>>>::TLabels),
}

impl Iterator for LabelIterator<'_> {
    type Item = Label;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LabelIterator::Single(ref mut s) => s.next(),
            LabelIterator::Chunk(ref mut c) => c.next(),
        }
    }
}
