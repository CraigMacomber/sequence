//! Hookup the [IndirectNode] and [UniformChunk] to [Nav] using [Forest] as the [Resolver].

use crate::{
    chunk::Chunk,
    forest::{self, ChunkId},
    indirect::{Child, NavChunk, NodeView},
    indirect_node::{IndirectChunk, IndirectNode},
    nav::{self, Resolver},
    node_id::{HasId, NodeId},
    tree::{Label, NodeNav, ParentInfo},
    uniform_chunk::{ChunkIterator, UniformChunkNode},
};

impl<'a> Chunk for &'a NavChunk {
    type View = NodeView<'a>;
    type Child = Child<'a>;
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<NodeView<'a>> {
        match self {
            NavChunk::Indirect(node) => node.get(first_id, id).map(NodeView::Indirect),
            NavChunk::Uniform(c) => c.get(first_id, id).map(NodeView::Uniform),
        }
    }
}

pub type Forest = forest::Forest<NavChunk>;

pub enum Expander<'a> {
    UniformChunkNode(ChunkIterator<'a>),
    IndirectView(NodeView<'a>),
    Empty,
}

impl<'a> Iterator for Expander<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Expander::UniformChunkNode(ref mut c) => c.next().map(NodeView::Uniform),
            Expander::IndirectView(ref mut c) => {
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
            Child::Indirect(id) => match self.find_nodes(id).unwrap() {
                NavChunk::Indirect(basic) => {
                    Expander::IndirectView(NodeView::Indirect(IndirectNode {
                        node: basic,
                        id: id.0,
                    }))
                }
                NavChunk::Uniform(chunk) => {
                    Expander::UniformChunkNode(ChunkIterator::View(UniformChunkNode {
                        view: chunk.view(id.0),
                        offset: 0,
                    }))
                }
            },
            Child::Uniform(chunk) => Expander::UniformChunkNode(ChunkIterator::View(chunk)),
        }
    }

    fn get_parent(&self, node: &NodeView<'a>) -> Option<ParentInfo<NodeView<'a>>> {
        match node {
            NodeView::Indirect(basic) => self.get_parent_from_chunk_id(ChunkId(basic.id)),
            NodeView::Uniform(chunk) => {
                // TODO: Performance: maybe avoid id lookup then node look up from id below?
                let id = chunk.get_id();
                let (chunk_id, chunk) = self.find_nodes_from_node(id).unwrap();
                match chunk {
                    NavChunk::Uniform(c) => {
                        let info = c.schema.lookup_schema(chunk_id.0, id).unwrap();
                        let parent = match info.parent.parent {
                            Some(x) => x,
                            None => {
                                return self.get_parent_from_chunk_id(*chunk_id);
                            }
                        };
                        Some(ParentInfo {
                            node: NodeView::Uniform(
                                c.get(chunk_id.0, chunk_id.0 + parent.0).unwrap(),
                            ),
                            label: parent.1,
                        })
                    }
                    _ => panic!(),
                }
            }
        }
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
    type TTraitChildren = <IndirectNode<'a> as NodeNav<ChunkId>>::TTraitChildren;
    type TLabels = LabelIterator<'a>;

    fn get_traits(&self) -> Self::TLabels {
        match self {
            NavChunk::Indirect(s) => LabelIterator::Single(s.get_traits()),
            NavChunk::Uniform(_) => LabelIterator::Empty,
        }
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        match self {
            NavChunk::Indirect(s) => s.get_trait(label),
            NavChunk::Uniform(_) => IndirectChunk::empty_trait(),
        }
    }
}

pub enum LabelIterator<'a> {
    Single(<IndirectNode<'a> as NodeNav<ChunkId>>::TLabels),
    Empty,
}

impl<'a> Iterator for LabelIterator<'a> {
    type Item = Label;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LabelIterator::Single(ref mut s) => s.next(),
            LabelIterator::Empty => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Def;

    #[test]
    fn it_works() {
        let mut forest = Forest::new();
        forest.insert(
            ChunkId(NodeId(5)),
            NavChunk::Indirect(IndirectChunk {
                def: Def(1),
                payload: None,
                traits: im_rc::HashMap::default(),
            }),
        );

        let _n = forest.find_node(NodeId(5)).unwrap();
        let _nav = forest.nav_from(NodeId(5)).unwrap();
        let n = forest.find_nodes(ChunkId(NodeId(5))).unwrap();
        let _n = Chunk::get(&n, NodeId(5), NodeId(5)).unwrap();
    }
}
