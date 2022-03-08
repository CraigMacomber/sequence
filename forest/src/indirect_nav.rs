//! Hookup [enum_chunk] to [nav] using [Forest] as the [Resolver].

use crate::{
    chunk::{Chunk, ChunkId},
    forest,
    indirect::enum_chunk,
    nav::{self, Resolver},
    node_id::{HasId, NodeId},
    tree::ParentInfo,
    uniform_chunk::ChunkIterator,
};

pub type Forest = forest::Forest<enum_chunk::Chunk>;

impl<'a> Resolver<enum_chunk::Node<'a>> for &'a Forest {
    type Child = enum_chunk::Child<'a>;

    type Iter = enum_chunk::Expander<'a>;

    fn expand(&self, chunk: Self::Child) -> Self::Iter {
        match chunk {
            enum_chunk::Child::Indirect(id) => {
                self.find_nodes(id).unwrap().top_level_nodes(id.0).into()
            }
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
                // Currently UniformNodes don't store a reference to the UniformChunk they are in (instead just the parts they need).
                // Since we need to actual root schema to do the parent lookup, recover the actual chunk from the Forest:
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

impl<TChunk> forest::Forest<TChunk>
where
    TChunk: Clone + PartialEq<TChunk>,
    for<'a> &'a TChunk: Chunk,
    for<'a> &'a Self: Resolver<<&'a TChunk as Chunk>::View>,
{
    pub fn nav_from(&self, id: NodeId) -> Option<nav::Nav<&Self, <&TChunk as Chunk>::View>> {
        self.find_node(id)
            .map(|view| nav::Nav::<&Self, <&TChunk as Chunk>::View>::new(self, view))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{indirect_node::IndirectChunk, tree::Def};

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
