//! Implementation of Node thats indirect, and supports multiple representations (chunk, and basic nodes)

use crate::{
    basic_indirect::BasicNode, chunk::ChunkOffset, forest::ChunkId, Def, HasId, Label, Node, NodeId,
};

pub enum Child<'a> {
    Id(ChunkId),
    Chunk(ChunkOffset<'a>),
}

pub enum TraitView<'a> {
    Basic(<&'a BasicNode<ChunkId> as Node<ChunkId>>::TTrait),
    Chunk(<ChunkOffset<'a> as Node<ChunkOffset<'a>>>::TTrait),
}

#[derive(Clone)]
pub struct BasicView<'a> {
    pub node: &'a BasicNode<ChunkId>,
    pub id: NodeId,
}

#[derive(Clone)]
pub enum NodeView<'a> {
    Single(BasicView<'a>),
    Chunk(ChunkOffset<'a>),
    // TODO: support undownloaded chunks blobs (find can return which blobs and at what offset the node is at)
    // TODO: support undownloaded subtrees that arn't chunks: find returns iterator of candidate trees using bloom filters
    // TODO: these types are write optimized. Consider supporting read/size optimized types (ex: using byte array instead of im's Vector)
    // TODO: maybe chunks referencing external subtrees (so they can have child references like payloads)
}

impl<'a> Node<Child<'a>> for NodeView<'a> {
    type TTrait = TraitView<'a>;

    type TTraitIterator = TraitIterator<'a>;

    fn get_def(&self) -> Def {
        match self {
            NodeView::Single(s) => s.node.get_def(),
            NodeView::Chunk(c) => c.get_def(),
        }
    }

    fn get_payload(&self) -> Option<crate::util::ImSlice> {
        match self {
            NodeView::Single(s) => s.node.get_payload(),
            NodeView::Chunk(c) => c.get_payload(),
        }
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        match self {
            NodeView::Single(s) => TraitIterator::Single(s.node.get_traits()),
            NodeView::Chunk(c) => TraitIterator::Chunk(c.get_traits()),
        }
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        match self {
            NodeView::Single(s) => TraitView::Basic(s.node.get_trait(label)),
            NodeView::Chunk(c) => TraitView::Chunk(c.get_trait(label)),
        }
    }
}

impl<'a> HasId for NodeView<'a> {
    fn get_id(&self) -> NodeId {
        match self {
            NodeView::Single(s) => s.id,
            NodeView::Chunk(c) => c.get_id(),
        }
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

pub enum TraitIterator<'a> {
    Single(<&'a BasicNode<ChunkId> as Node<ChunkId>>::TTraitIterator),
    Chunk(<ChunkOffset<'a> as Node<ChunkOffset<'a>>>::TTraitIterator),
}

impl<'a> Iterator for TraitIterator<'a> {
    type Item = Label;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TraitIterator::Single(ref mut s) => s.next(),
            TraitIterator::Chunk(ref mut c) => c.next(),
        }
    }
}
