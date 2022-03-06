//! Generic Nav which handles child id indirection via a Resolver which returns an iterable of Nodes for an Id.

use crate::{
    node_id::{HasId, NodeId},
    tree::{Def, Label, NodeData, NodeNav, ParentInfo},
};

/// Chunk resolver
pub trait Resolver<Node>: Copy {
    type ChunkId;
    type Iter: Iterator<Item = Node>;
    fn expand(&self, chunk: Self::ChunkId) -> Self::Iter;
    fn get_parent(&self, node: &Node) -> Option<ParentInfo<Node>>;
}

#[derive(Clone)]
pub struct Nav<R, TNode> {
    resolver: R,
    view: TNode,
}

impl<R, TNode> Nav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: HasId,
{
    pub fn new(resolver: R, view: TNode) -> Self {
        Nav { resolver, view }
    }

    pub fn parent(&self) -> Option<ParentInfo<Self>> {
        self.resolver.get_parent(&self.view).map(|p| ParentInfo {
            node: Self::new(self.resolver, p.node),
            label: p.label,
        })
    }
}

impl<R, TNode: HasId> HasId for Nav<R, TNode> {
    fn get_id(&self) -> NodeId {
        self.view.get_id()
    }
}

pub struct TraitNav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: NodeNav<<R as Resolver<TNode>>::ChunkId>,
{
    resolver: R,
    view: <TNode as NodeNav<<R as Resolver<TNode>>::ChunkId>>::TTraitChildren,
    pending: Option<<R as Resolver<TNode>>::Iter>,
}

impl<R, TNode> NodeNav<Nav<R, TNode>> for Nav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: NodeNav<<R as Resolver<TNode>>::ChunkId>,
{
    type TTraitChildren = TraitNav<R, TNode>;
    type TLabels = TNode::TLabels;

    fn get_traits(&self) -> Self::TLabels {
        self.view.get_traits()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        TraitNav {
            view: self.view.get_trait(label),
            resolver: self.resolver,
            pending: None,
        }
    }
}

impl<R, TNode> NodeData for Nav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: NodeData,
{
    fn get_def(&self) -> Def {
        self.view.get_def()
    }

    fn get_payload(&self) -> Option<crate::util::ImSlice> {
        self.view.get_payload()
    }
}

impl<R, TNode> Iterator for TraitNav<R, TNode>
where
    R: Resolver<TNode>,
    TNode: NodeNav<<R as Resolver<TNode>>::ChunkId>,
{
    type Item = Nav<R, TNode>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut chunks) = self.pending {
            match chunks.next() {
                Some(chunk) => {
                    return Some(Nav {
                        resolver: self.resolver,
                        view: chunk,
                    });
                }
                None => self.pending = None,
            }
        }

        match self.view.next() {
            Some(id) => {
                let mut iter = self.resolver.expand(id);
                let result = iter.next().unwrap(); // Chunk must not be empty
                self.pending = Some(iter);
                Some(Nav {
                    view: result,
                    resolver: self.resolver,
                })
            }
            None => None,
        }
    }
}
