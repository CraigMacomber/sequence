//! Hookup to Generic Nav. This takes indirect_dynamic::NodeView, and wraps it with a recursive nav type that handles child lookup using Forest.

use crate::{
    basic_indirect::BasicNode,
    chunk::{Chunk, ChunkIterator, ChunkOffset},
    forest::{self, ChunkId},
    indirect_dynamic::{Child, NodeView},
    nav::{Nav, Resolver},
    Node, NodeId,
};

#[derive(Clone)]
pub enum NavChunk {
    Single(BasicNode<NodeId, ChunkId>),
    Chunk(Chunk<NodeId>),
}

impl<'a> forest::Nodes for &'a NavChunk {
    type View = NodeView<'a>;
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

impl<'a> Resolver<NodeView<'a>> for &'a Forest {
    type ChunkId = Child<'a>;

    type Iter = Expander<'a>;

    fn expand(&self, chunk: Self::ChunkId) -> Self::Iter {
        match chunk {
            Child::Id(id) => match self.find_nodes(id).unwrap() {
                NavChunk::Single(basic) => Expander::Single(NodeView::Single(basic)),
                NavChunk::Chunk(chunk) => Expander::Chunk(ChunkIterator::View(ChunkOffset {
                    view: chunk.view(),
                    offset: 0,
                })),
            },
            Child::Chunk(chunk) => Expander::Chunk(ChunkIterator::View(chunk)),
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
    use crate::{Def, Label};

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
