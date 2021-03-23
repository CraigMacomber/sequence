//! Specialized Nav

use crate::{
    basic_indirect::BasicNode,
    chunk::{Chunk, ChunkIterator, ChunkOffset},
    forest::{self, ChunkId, Forest},
    indirect_dynamic::{NodeView, TraitView},
    Def, Label, Node, NodeId,
};

#[derive(Clone)]
pub enum NavChunk {
    Single(BasicNode<NodeId, ChunkId>),
    Chunk(Chunk<NodeId>),
}

impl<'a> forest::Nodes<NodeView<'a>> for &'a NavChunk {
    fn first_id(&self) -> NodeId {
        match self {
            NavChunk::Single(n) => n.get_id(),
            NavChunk::Chunk(c) => c.first_id,
        }
    }

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

pub struct Nav<'a> {
    forest: &'a Forest<NavChunk, NodeView<'a>>,
    view: NodeView<'a>,
}
pub struct TraitNav<'a> {
    forest: &'a Forest<NavChunk, NodeView<'a>>,
    view: TraitView<'a>,
}

impl<'a> Node<Nav<'a>, NodeId> for Nav<'a> {
    type TTrait = TraitNav<'a>;

    type TTraitIterator = Vec<Label>; // TODO: maybe keep laziness?

    fn get_id(&self) -> NodeId {
        self.view.get_id()
    }

    fn get_def(&self) -> Def {
        self.view.get_def()
    }

    fn get_payload(&self) -> Option<crate::util::ImSlice> {
        self.view.get_payload()
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        self.view.get_traits().collect()
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        TraitNav {
            view: self.view.get_trait(label),
            forest: self.forest,
        }
    }
}

impl<'a> Iterator for TraitNav<'a> {
    type Item = Nav<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut c) = self.view.chunk {
            let r = c.next();
            match r {
                Some(rr) => {
                    return Some(Nav {
                        forest: self.forest,
                        view: NodeView::Chunk(rr),
                    });
                }
                None => {
                    self.view.chunk = None;
                }
            }
        }
        if let Some(ref mut c) = self.view.basic {
            let r = c.next();
            match r {
                Some(rr) => {
                    let nodes = self.forest.find_nodes(rr).unwrap();
                    match nodes {
                        NavChunk::Single(z) => {
                            return Some(Nav {
                                forest: self.forest,
                                view: NodeView::Single(z),
                            });
                        }
                        NavChunk::Chunk(z) => {
                            let xx = z.view();
                            let mut iterator = ChunkIterator::View(ChunkOffset {
                                offset: 0,
                                view: xx,
                            });
                            let result = iterator.next().unwrap(); // Fails if chunk is empty
                            self.view.chunk = Some(iterator);
                            return Some(Nav {
                                forest: self.forest,
                                view: NodeView::Chunk(result),
                            });
                        }
                    }
                }
                None => {
                    self.view.chunk = None;
                }
            }
        }
        None
    }
}

// generic Nav compat

// impl<'a> NavChunk<NodeView<'a>> for &'a Nodes {
//     type Iter = ChunkOrViewIterator<'a>;

//     fn iter(&self) -> Self::Iter {
//         self.
//         todo!()
//     }
// }

// enum ChunkOrViewIterator<'a> {
//     Single(Option<NodeView<'a>>),
//     Chunk(ChunkIterator<'a, NodeId>),
// }

// impl<'a> Iterator for ChunkOrViewIterator<'a> {
//     type Item = ChunkOrView<NodeView<'a>>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             ChunkOrViewIterator::Single(ref mut s) => s.take().map(|a| ChunkOrView::Single(a)),
//             ChunkOrViewIterator::Chunk(ref mut c) => c.next().map(|a| ChunkOrView::Chunk(a)),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forest::Forest;

    #[test]
    fn it_works() {
        let mut forest: Forest<NavChunk, _> = Forest::new();
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

        let nav = Nav {
            forest: &forest,
            view: n,
        };

        let children: Vec<_> = nav.get_trait(Label(9)).collect();
        assert!(children.len() == 0);

        let n = forest.find_nodes(ChunkId(NodeId(5))).unwrap();
        let n = forest::Nodes::get(&n, NodeId(5)).unwrap();
        assert!(n.get_def().0 == 1);
    }
}
