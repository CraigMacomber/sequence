//! Utilities for viewing trees of mixed [crate::basic_indirect::BasicNode] and [crate::chunk::UniformChunk].
//! where the children of the basic nodes are [ChunkId]s which correspond to either a chunk, or a BasicNode.
//!
//! The [crate::Node] implemented here is abstracts this as children are which are either Nodes, or ChunkIds.
//! When traversing within a chunk, no indirection is involved, looking up an Id is only required when traversing a BasicNode.
//!
//! Abstracting away this difference
//! (and dealing with the fact that a trait may contain a mix of chunks and basic nodes, and the chunks might contain multiple top level nodes)
//! is done by [crate::indirect_nav] which wraps this node in a Node implementation up with a forest using [crate::nav::Nav].

use derive_more::From;
use enum_dispatch::enum_dispatch;

use crate::{
    forest::ChunkId,
    indirect_node::BasicView,
    tree::{Label, NodeNav},
    uniform_chunk::ChunkOffset,
};

/// Child type for the [crate::Node].
#[derive(From)]
pub enum Child<'a> {
    /// Parent is a [crate::basic_indirect::BasicNode]: may resolve to either a [crate::basic_indirect::BasicView] or a [crate::chunk::UniformChunk]
    Id(ChunkId),
    /// A node within a [crate::chunk::UniformChunk]: can the the child of another node within the same chunk.
    Chunk(ChunkOffset<'a>),
}

#[derive(From)]
pub enum TraitView<'a> {
    Basic(<BasicView<'a> as NodeNav<ChunkId>>::TTraitChildren),
    Chunk(<ChunkOffset<'a> as NodeNav<ChunkOffset<'a>>>::TTraitChildren),
}

// enum_dispatch does not work with this use of NodeNav, so define another more specific trait instead.
#[enum_dispatch]
pub trait DynView<'a> {
    fn get_traits(&self) -> LabelIterator<'a>;
    fn get_trait(&self, label: Label) -> TraitView<'a>;
}

macro_rules! impl_DynView {
    ($name:ident, $child:ty) => {
        impl<'a> DynView<'a> for $name<'a> {
            fn get_traits(&self) -> LabelIterator<'a> {
                <Self as crate::tree::NodeNav<$child>>::get_traits(self).into()
            }

            fn get_trait(&self, label: Label) -> TraitView<'a> {
                <Self as crate::tree::NodeNav<$child>>::get_trait(self, label).into()
            }
        }
    };
}

impl_DynView!(ChunkOffset, ChunkOffset<'a>);
impl_DynView!(BasicView, ChunkId);

// impl<'a, T, TChild> DynView<'a> for T
// where
//     T: NodeNav<TChild>,
//     TChild: Into<Child<'a>>,
// {
//     fn get_traits(&self) -> LabelIterator<'a> {
//         <Self as NodeNav<TChild>>::get_traits(self).into()
//     }

//     fn get_trait(&self, label: Label) -> TraitView<'a> {
//         <Self as NodeNav<TChild>>::get_trait(self, label).into()
//     }
// }

// impl<'a, T, TChild> T
// where
//     T: NodeNav<TChild>,
//     TChild: Into<Child<'a>>,
// {
// }
#[enum_dispatch(HasId, DynView, NodeData)] // NodeNav
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
            TraitView::Basic(ref mut c) => c.next().map(|id| id.into()),
            TraitView::Chunk(ref mut c) => c.next().map(|c| c.into()),
        }
    }
}

#[derive(From)]
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
