use crate::{
    basic_indirect::BasicNode,
    basic_indirect::BasicTrait,
    chunk::{Chunk, ChunkOffset, ChunkView},
    forest::{self, ChunkId, Forest},
    Def, Label, Node, NodeId, Trait,
};

pub enum Nodes {
    Single(BasicNode<NodeId, ChunkId>),
    Chunk(Chunk<NodeId>),
}

pub enum TraitView<'a> {
    Single(&'a BasicTrait<ChunkId>),
    Chunk(ChunkView<'a, NodeId>),
}

impl<'a> forest::Nodes<NodeView<'a>> for &'a Nodes {
    fn first_id(&self) -> NodeId {
        match self {
            Nodes::Single(n) => n.get_id(),
            Nodes::Chunk(c) => c.first_id,
        }
    }

    fn get(&self, id: NodeId) -> Option<NodeView<'a>> {
        match self {
            Nodes::Single(n) => {
                if n.get_id() == id {
                    Some(NodeView::Single(n))
                } else {
                    None
                }
            }
            Nodes::Chunk(c) => c.lookup(id).map(|x| NodeView::Chunk(x)),
        }
    }
}

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

    fn get_trait(&self, label: Label) -> Option<Self::TTrait> {
        match self {
            NodeView::Single(s) => s.get_trait(label).map(|t| TraitView::Single(t)),
            NodeView::Chunk(c) => c.get_trait(label).map(|t| TraitView::Chunk(t)),
        }
    }
}

impl<'a> Trait<ChunkId> for TraitView<'a> {
    fn get_count(&self) -> usize {
        match self {
            TraitView::Single(s) => s.get_count(),
            TraitView::Chunk(c) => c.get_count(),
        }
    }

    fn get_child(&self, index: usize) -> ChunkId {
        match self {
            TraitView::Single(s) => s.get_child(index),
            // TODO: wrapper around this Node impl is going to have to re-get this child. Thats inefficient.
            TraitView::Chunk(c) => ChunkId(c.get_child(index).get_id()),
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
        todo!()
    }
}

// make generic nav: Node<T> + Resolver<T> -> Nav: Node<Nav>

pub struct Nav<'a> {
    forest: &'a Forest<&'a Nodes, NodeView<'a>>,
    view: NodeView<'a>,
}
pub struct TraitNav<'a> {
    forest: &'a Forest<&'a Nodes, NodeView<'a>>,
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

    fn get_trait(&self, label: Label) -> Option<Self::TTrait> {
        self.view.get_trait(label).map(|view| TraitNav {
            view,
            forest: self.forest,
        })
    }
}

impl<'a> Trait<Nav<'a>> for TraitNav<'a> {
    fn get_count(&self) -> usize {
        match &self.view {
            TraitView::Single(d) => d
                .children
                .iter()
                .map(|n| match self.forest.find_nodes(*n).unwrap() {
                    Nodes::Single(_) => 1,
                    Nodes::Chunk(c) => c.get_count(),
                })
                .sum(),
            TraitView::Chunk(c) => c.get_count(),
        }
    }

    fn get_child(&self, index: usize) -> Nav<'a> {
        match &self.view {
            TraitView::Single(d) => {
                let mut count: usize = 0;
                for n in &d.children {
                    match self.forest.find_nodes(*n).unwrap() {
                        Nodes::Single(d) => {
                            if count == index {
                                return Nav {
                                    forest: self.forest,
                                    view: NodeView::Single(d),
                                };
                            }
                            count += 1;
                        }
                        Nodes::Chunk(c) => {
                            let more = c.get_count();
                            if count + more > index {
                                return Nav {
                                    forest: self.forest,
                                    view: NodeView::Chunk(ChunkOffset {
                                        view: c.view(),
                                        offset: (index - count) as u32,
                                    }),
                                };
                            }
                            count += more;
                        }
                    };
                }
                panic!("trait index out of range")
            }
            TraitView::Chunk(c) => Nav {
                forest: self.forest,
                view: NodeView::Chunk(c.get_child(index)),
            },
        }
    }
}
