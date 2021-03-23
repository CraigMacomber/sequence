//! Hookup to Generic Nav

use crate::{
    basic_indirect::BasicNode,
    chunk::{Chunk, ChunkIterator, ChunkOffset},
    forest::{self, ChunkId},
    nav::{Nav, Resolver},
    Def, Label, Node, NodeId,
};

#[derive(Clone)]
pub enum NavChunk {
    Single(BasicNode<NodeId, ChunkId>),
    Chunk(Chunk<NodeId>),
}

impl<'a> forest::Nodes for &'a NavChunk {
    type View = NodeView<'a>;
    // fn first_id(&self) -> NodeId {
    //     match self {
    //         NavChunk::Single(n) => n.get_id(),
    //         NavChunk::Chunk(c) => c.first_id,
    //     }
    // }

    fn get(&self, id: NodeId) -> Option<NodeView<'a>> {
        match self {
            NavChunk::Single(n) => {
                if n.get_id() == id {
                    Some(NodeView::Single(n))
                } else {
                    None
                }
            }
            NavChunk::Chunk(c) => c.lookup(id).map(|x| NodeView::Chunk(x)),
        }
    }
}

type Forest = forest::Forest<NavChunk>;
pub enum Expander<'a> {
    Chunk(ChunkIterator<'a, NodeId>),
    Single(NodeView<'a>),
    Empty,
}

impl<'a> Iterator for Expander<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Expander::Chunk(ref mut c) => c.next().map(|chunk| NodeView::Chunk(chunk)),
            Expander::Single(ref mut c) => {
                let result = c.clone();
                *self = Expander::Empty;
                Some(result)
            }
            Expander::Empty => None,
        }
    }
}

pub enum ResolverId<'a> {
    Id(ChunkId),
    Chunk(ChunkOffset<'a, NodeId>),
}

impl<'a> Resolver<NodeView<'a>> for &'a Forest {
    type ChunkId = ResolverId<'a>;

    type Iter = Expander<'a>;

    fn expand(&self, chunk: Self::ChunkId) -> Self::Iter {
        match chunk {
            ResolverId::Id(id) => match self.find_nodes(id).unwrap() {
                NavChunk::Single(basic) => Expander::Single(NodeView::Single(basic)),
                NavChunk::Chunk(chunk) => Expander::Chunk(ChunkIterator::View(ChunkOffset {
                    view: chunk.view(),
                    offset: 0,
                })),
            },
            ResolverId::Chunk(chunk) => Expander::Chunk(ChunkIterator::View(chunk)),
        }
    }
}

impl Forest {
    pub fn nav_from(&self, id: NodeId) -> Option<Nav<&Self, NodeView>> {
        self.find_node(id).map(|view| Nav::new(self, view))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut forest = Forest::new();
        forest.map.insert(
            ChunkId(NodeId(5)),
            NavChunk::Single(BasicNode {
                def: Def(1),
                id: NodeId(5),
                payload: None,
                traits: im_rc::HashMap::new(),
            }),
        );

        let n = forest.find_node(NodeId(5)).unwrap();
        assert!(n.get_def().0 == 1);

        let nav = forest.nav_from(NodeId(5)).unwrap();

        let children: Vec<_> = nav.get_trait(Label(9)).collect();
        assert!(children.len() == 0);

        let n = forest.find_nodes(ChunkId(NodeId(5))).unwrap();
        let n = forest::Nodes::get(&n, NodeId(5)).unwrap();
        assert!(n.get_def().0 == 1);
    }
}

//// indirect dynamic 2

pub enum TraitView<'a> {
    Basic(<&'a BasicNode<NodeId, ChunkId> as Node<ChunkId, NodeId>>::TTrait),
    Chunk(<ChunkOffset<'a, NodeId> as Node<ChunkOffset<'a, NodeId>, NodeId>>::TTrait),
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

impl<'a> Node<ResolverId<'a>, NodeId> for NodeView<'a> {
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
            NodeView::Single(s) => TraitView::Basic(s.get_trait(label)),
            NodeView::Chunk(c) => TraitView::Chunk(c.get_trait(label)),
        }
    }
}

impl<'a> Iterator for TraitView<'a> {
    type Item = ResolverId<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TraitView::Basic(ref mut c) => c.next().map(|id| ResolverId::Id(id)),
            TraitView::Chunk(ref mut c) => c.next().map(|c| ResolverId::Chunk(c)),
        }
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
