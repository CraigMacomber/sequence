//! Hookup the [BasicNode] to [Nav] using [Forest] as the [Resolver].

use crate::{
    basic_indirect::BasicNode,
    chunk::{Chunk, ChunkIterator, ChunkOffset},
    forest::{self, ChunkId, ParentInfo},
    indirect::{BasicView, Child, NodeView},
    nav::{self, ParentResolver, Resolver},
    HasId, Label, NodeId, NodeNav,
};

/// Tree data, stored in the forest, keyed by the first id in the chunk.
#[derive(Clone, PartialEq)]
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

impl<'a> ParentResolver<NodeView<'a>> for &'a Forest {
    fn get_parent(&self, node: &NodeView<'a>) -> Option<ParentInfo<NodeView<'a>>> {
        match node {
            NodeView::Single(basic) => self.get_parent_from_chunk_id(ChunkId(basic.id)),
            NodeView::Chunk(chunk) => {
                // TODO: Performance: maybe avoid id lookup then node look up from id below?
                let id = chunk.get_id();
                let (chunk_id, chunk) = self.find_nodes_from_node(id).unwrap();
                match chunk {
                    NavChunk::Single(_) => {
                        panic!()
                    }
                    NavChunk::Chunk(c) => {
                        let info = c.schema.lookup_schema(chunk_id.0, id).unwrap();
                        let parent = match info.parent.parent {
                            Some(x) => x,
                            None => {
                                return self.get_parent_from_chunk_id(*chunk_id);
                            }
                        };
                        Some(ParentInfo {
                            node: NodeView::Chunk(
                                c.lookup(chunk_id.0, chunk_id.0 + parent.0).unwrap(),
                            ),
                            label: parent.1,
                        })
                    }
                }
            }
        }
    }
}

impl Forest {
    fn get_parent_from_chunk_id(&self, id: ChunkId) -> Option<ParentInfo<NodeView>> {
        self.get_parent_data().get(&id).map(|x| ParentInfo {
            node: self.find_node(x.node.0).unwrap(),
            label: x.label,
        })
    }
}

pub type Nav<'a> = nav::Nav<&'a Forest, NodeView<'a>>;

impl Forest {
    pub fn nav_from(&self, id: NodeId) -> Option<Nav> {
        self.find_node(id).map(|view| Nav::new(self, view))
    }
}

/// For parent info: Allow viewing the tree of chunks as Node
/// TODO: maybe there are other uses for this? Might be able to simplify code elsewhere.

impl<'a> NodeNav<ChunkId> for &'a NavChunk {
    type TTraitChildren = TraitView<'a>;
    type TLabels = TraitIterator<'a>;

    fn get_traits(&self) -> Self::TLabels {
        match self {
            NavChunk::Single(s) => TraitIterator::Single(s.get_traits()),
            NavChunk::Chunk(_) => TraitIterator::Empty,
        }
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        match self {
            NavChunk::Single(s) => TraitView::Basic(s.get_trait(label)),
            NavChunk::Chunk(_) => TraitView::Empty,
        }
    }
}

pub enum TraitView<'a> {
    Basic(<&'a BasicNode<ChunkId> as NodeNav<ChunkId>>::TTraitChildren),
    Empty,
}

impl<'a> Iterator for TraitView<'a> {
    type Item = ChunkId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TraitView::Basic(ref mut c) => c.next(),
            TraitView::Empty => None,
        }
    }
}

pub enum TraitIterator<'a> {
    Single(<&'a BasicNode<ChunkId> as NodeNav<ChunkId>>::TLabels),
    Empty,
}

impl<'a> Iterator for TraitIterator<'a> {
    type Item = Label;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TraitIterator::Single(ref mut s) => s.next(),
            TraitIterator::Empty => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Def;

    #[test]
    fn it_works() {
        let mut forest = Forest::new();
        forest.insert(
            ChunkId(NodeId(5)),
            NavChunk::Single(BasicNode {
                def: Def(1),
                payload: None,
                traits: im_rc::HashMap::default(),
            }),
        );

        let _n = forest.find_node(NodeId(5)).unwrap();
        let _nav = forest.nav_from(NodeId(5)).unwrap();
        let n = forest.find_nodes(ChunkId(NodeId(5))).unwrap();
        let _n = forest::Nodes::get(&n, NodeId(5), NodeId(5)).unwrap();
    }
}
