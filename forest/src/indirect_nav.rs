//! Hookup the [IndirectNode] and [UniformChunk] to [Nav] using [Forest] as the [Resolver].

use crate::{
    chunk::Chunk,
    forest::{self, ChunkId},
    indirect::enum_chunk,
    indirect_node::{IndirectChunk, IndirectNode},
    nav::{self, Resolver},
    node_id::{HasId, NodeId},
    tree::{Label, NodeNav, ParentInfo},
    uniform_chunk::ChunkIterator,
};

pub type Forest = forest::Forest<enum_chunk::Chunk>;

impl<'a> Resolver<enum_chunk::Node<'a>> for &'a Forest {
    type ChunkId = enum_chunk::Child<'a>;

    type Iter = enum_chunk::Expander<'a>;

    fn expand(&self, chunk: Self::ChunkId) -> Self::Iter {
        match chunk {
            enum_chunk::Child::Indirect(id) => match self.find_nodes(id).unwrap() {
                enum_chunk::Chunk::Indirect(chunk) => chunk.top_level_nodes(id.0).into(),
                enum_chunk::Chunk::Uniform(chunk) => chunk.top_level_nodes(id.0).into(),
            },
            enum_chunk::Child::Uniform(chunk) => {
                enum_chunk::Expander::Uniform(ChunkIterator::View(chunk))
            }
        }
    }

    fn get_parent(&self, node: &enum_chunk::Node<'a>) -> Option<ParentInfo<enum_chunk::Node<'a>>> {
        match node {
            enum_chunk::Node::Indirect(basic) => self.get_parent_from_chunk_id(ChunkId(basic.id)),
            enum_chunk::Node::Uniform(chunk) => {
                // TODO: Performance: maybe avoid id lookup then node look up from id below?
                let id = chunk.get_id();
                let (chunk_id, chunk) = self.find_nodes_from_node(id).unwrap();
                match chunk {
                    enum_chunk::Chunk::Uniform(c) => {
                        let info = c.schema.lookup_schema(chunk_id.0, id).unwrap();
                        let parent = match info.parent.parent {
                            Some(x) => x,
                            None => {
                                return self.get_parent_from_chunk_id(*chunk_id);
                            }
                        };
                        Some(ParentInfo {
                            node: enum_chunk::Node::Uniform(
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

pub type Nav<'a> = nav::Nav<&'a Forest, enum_chunk::Node<'a>>;

impl Forest {
    pub fn nav_from(&self, id: NodeId) -> Option<Nav> {
        self.find_node(id).map(|view| Nav::new(self, view))
    }
}

/// For parent info: Allow viewing the tree of chunks as Node
/// TODO: maybe there are other uses for this? Might be able to simplify code elsewhere.

impl<'a> NodeNav<ChunkId> for &'a enum_chunk::Chunk {
    type TTraitChildren = <IndirectNode<'a> as NodeNav<ChunkId>>::TTraitChildren;
    type TLabels = LabelIterator<'a>;

    fn get_traits(&self) -> Self::TLabels {
        match self {
            enum_chunk::Chunk::Indirect(s) => LabelIterator::Single(s.get_traits()),
            enum_chunk::Chunk::Uniform(_) => LabelIterator::Empty,
        }
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        match self {
            enum_chunk::Chunk::Indirect(s) => s.get_trait(label),
            enum_chunk::Chunk::Uniform(_) => IndirectChunk::empty_trait(),
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
            IndirectChunk {
                def: Def(1),
                payload: None,
                traits: im_rc::HashMap::default(),
            }
            .into(),
        );

        let _n = forest.find_node(NodeId(5)).unwrap();
        let _nav = forest.nav_from(NodeId(5)).unwrap();
        let n = forest.find_nodes(ChunkId(NodeId(5))).unwrap();
        let _n = Chunk::get(&n, NodeId(5), NodeId(5)).unwrap();
    }
}
