//! Generic Nav which handles child id indirection via a Resolver which returns an iterable of Nodes for an Id.

use crate::{Def, Label, Node, NodeId};

/// Chunk resolver
pub trait Resolver<Node>: Copy {
    type ChunkId;
    type Iter: Iterator<Item = Node>;
    fn expand(&self, chunk: Self::ChunkId) -> Self::Iter;
}

pub struct Nav<R, TNode> {
    resolver: R,
    view: TNode,
}

impl<R, TNode> Nav<R, TNode> {
    pub fn new(resolver: R, view: TNode) -> Self {
        Nav { resolver, view }
    }
}
pub struct TraitNav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: Node<<R as Resolver<TNode>>::ChunkId, NodeId>,
{
    resolver: R,
    view: <TNode as Node<<R as Resolver<TNode>>::ChunkId, NodeId>>::TTrait,
    pending: Option<<R as Resolver<TNode>>::Iter>,
}

impl<R, TNode> Node<Nav<R, TNode>, NodeId> for Nav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: Node<<R as Resolver<TNode>>::ChunkId, NodeId>,
{
    type TTrait = TraitNav<R, TNode>;

    type TTraitIterator = TNode::TTraitIterator;

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
        self.view.get_traits()
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        TraitNav {
            view: self.view.get_trait(label),
            resolver: self.resolver,
            pending: None,
        }
    }
}

impl<R, TNode> Iterator for TraitNav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: Node<<R as Resolver<TNode>>::ChunkId, NodeId>,
{
    type Item = Nav<R, TNode>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pending {
            Some(ref mut chunks) => match chunks.next() {
                Some(chunk) => {
                    return Some(Nav {
                        resolver: self.resolver,
                        view: chunk,
                    });
                }
                None => self.pending = None,
            },
            None => {}
        }

        match self.view.next() {
            Some(id) => {
                let iter = self.resolver.expand(id);
                let result = self.next().unwrap(); // Chunk must not be empty
                self.pending = Some(iter);
                return Some(result);
            }
            None => return None,
        }
    }
}
