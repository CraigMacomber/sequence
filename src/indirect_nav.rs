//! Hookup to indirect node to Nav.
//! This takes indirect::NodeView, and wraps it with a recursive nav type that handles child lookup using Forest.
//! uses `nav` to do this.

use crate::{
    basic_indirect::BasicNode,
    chunk::{Chunk, ChunkIterator, ChunkOffset},
    forest::{self, ChunkId},
    indirect::{BasicView, Child, NodeView},
    nav::{Nav, Resolver},
    NodeId,
};

/// Tree data, stored in the forest, keyed by the first id in the chunk.
#[derive(Clone)]
pub enum NavChunk {
    Single(BasicNode<ChunkId>),
    Chunk(Chunk),
}

impl<'a> forest::Nodes for &'a NavChunk {
    type View = NodeView<'a>;
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<NodeView<'a>> {
        match self {
            NavChunk::Single(node) => {
                if first_id == id {
                    Some(NodeView::Single(BasicView { node, id }))
                } else {
                    None
                }
            }
            NavChunk::Chunk(c) => c.lookup(first_id, id).map(|x| NodeView::Chunk(x)),
        }
    }
}

pub type Forest = forest::Forest<NavChunk>;
pub enum Expander<'a> {
    Chunk(ChunkIterator<'a>),
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
                NavChunk::Single(basic) => Expander::Single(NodeView::Single(BasicView {
                    node: basic,
                    id: id.0,
                })),
                NavChunk::Chunk(chunk) => Expander::Chunk(ChunkIterator::View(ChunkOffset {
                    view: chunk.view(id.0),
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
    use crate::{Def, Label, Node};

    #[test]
    fn it_works() {
        let mut forest = Forest::new();
        forest.insert(
            ChunkId(NodeId(5)),
            NavChunk::Single(BasicNode {
                def: Def(1),
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
        let n = forest::Nodes::get(&n, NodeId(5), NodeId(5)).unwrap();
        assert!(n.get_def().0 == 1);
    }
}
