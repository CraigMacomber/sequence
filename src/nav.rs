//! Allow viewing a forest of chunks as a tree

use crate::{
    forest::{ChunkId, Forest, Nodes},
    Def, Label, Node, NodeId,
};

// make generic nav: Node<T> + Resolver<T> -> Nav: Node<Nav>

pub trait NavChunk<View>: Nodes<View> {
    // iterate the roots of this chunk. This is a subset of ids owned by it.
    type Iter: Iterator<Item = ChunkOrView<View>>;
    fn iter(&self) -> Self::Iter;
}

pub enum ChunkOrView<View> {
    Single(View),
    Chunk(ChunkId),
}

pub struct Nav<'a, N, V>
where
    &'a N: Nodes<V>,
{
    forest: &'a Forest<&'a N, V>,
    view: V,
}
pub struct TraitNav<'a, N, V, T>
where
    &'a N: Nodes<V>,
{
    forest: &'a Forest<&'a N, V>,
    view: T,
}

impl<'a, N, V> Node<Nav<'a, N, V>, NodeId> for Nav<'a, N, V>
where
    &'a N: Nodes<V>,
    V: Node<ChunkId, NodeId>,
{
    type TTrait = TraitNav<'a, N, V, V::TTrait>;

    type TTraitIterator = V::TTraitIterator;

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
            forest: self.forest,
        }
    }
}

impl<'a, N, V, T> Iterator for TraitNav<'a, N, V, T>
where
    &'a N: Nodes<V>,
    V: Node<ChunkId, NodeId>,
{
    type Item = Nav<'a, N, V>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
