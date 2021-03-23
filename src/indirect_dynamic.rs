use crate::{
    basic_indirect::BasicNode, chunk::ChunkOffset, forest::ChunkId, Def, Label, Node, NodeId,
};

pub struct TraitView<'a> {
    // Iterate this first
    pub basic: Option<<&'a BasicNode<NodeId, ChunkId> as Node<ChunkId, NodeId>>::TTrait>,
    // Then this
    pub chunk: Option<<ChunkOffset<'a, NodeId> as Node<ChunkOffset<'a, NodeId>, NodeId>>::TTrait>,
}

#[derive(Clone)]
pub enum NodeView<'a> {
    Single(&'a BasicNode<NodeId, ChunkId>),
    Chunk(ChunkOffset<'a, NodeId>),
    // TODO: support undownloaded chunks blobs (find can return which blobs and at what offset the node is at)
    // TODO: support undownloaded subtrees that arn't chunks: find returns iterator of candidate trees using bloom filters
    // TODO: these types are write optimized. Consider supporting read/size optimized types (ex: using byte array instead of im's Vector)
    // TODO: maybe chunks referencing external subtrees (so they can have child references like payloads)
}

impl<'a> Node<ChunkId, NodeId> for NodeView<'a> {
    type TTrait = TraitView<'a>;

    type TTraitIterator = TraitIterator<'a>;

    fn get_id(&self) -> NodeId {
        match self {
            NodeView::Single(s) => s.get_id(),
            NodeView::Chunk(c) => c.get_id(),
        }
    }

    fn get_def(&self) -> Def {
        match self {
            NodeView::Single(s) => s.get_def(),
            NodeView::Chunk(c) => c.get_def(),
        }
    }

    fn get_payload(&self) -> Option<crate::util::ImSlice> {
        match self {
            NodeView::Single(s) => s.get_payload(),
            NodeView::Chunk(c) => c.get_payload(),
        }
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        match self {
            NodeView::Single(s) => TraitIterator::Single(s.get_traits()),
            NodeView::Chunk(c) => TraitIterator::Chunk(c.get_traits()),
        }
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        match self {
            NodeView::Single(s) => TraitView {
                basic: Some(s.get_trait(label)),
                chunk: None,
            },
            NodeView::Chunk(c) => TraitView {
                basic: None,
                chunk: Some(c.get_trait(label)),
            },
        }
    }
}

impl<'a> Iterator for TraitView<'a> {
    type Item = ChunkId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut c) = self.chunk {
            return c.next().map(|i| ChunkId(i.get_id()));
        }
        if let Some(ref mut c) = self.basic {
            return c.next();
        }
        None
    }
}

pub enum TraitIterator<'a> {
    Single(<&'a BasicNode<NodeId, ChunkId> as Node<ChunkId, NodeId>>::TTraitIterator),
    Chunk(<ChunkOffset<'a, NodeId> as Node<ChunkOffset<'a, NodeId>, NodeId>>::TTraitIterator),
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
