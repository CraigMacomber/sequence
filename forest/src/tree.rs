//! Core types of the tree abstraction.

use enum_dispatch::enum_dispatch;

use crate::indirect::NodeView;
use crate::util::ImSlice;

pub type IdBase = u128;

#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy)]
pub struct Def(pub IdBase);
#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy, Debug)]
pub struct Label(pub IdBase);

/// Navigation part of Node
pub trait NodeNav<TChild> {
    /// For iterating children within a trait.
    type TTraitChildren: Iterator<Item = TChild>;
    /// For iterating the set of trait labels for non-empty traits..
    type TLabels: Iterator<Item = Label>;

    // TODO: Performance: walking traits could be faster if this returned a reference to the trait not just the labels (saves a map lookup)
    fn get_traits(&self) -> Self::TLabels;
    fn get_trait(&self, label: Label) -> Self::TTraitChildren;
}

/// Tree Node.
/// Combines navigation with data (def and payload)
#[enum_dispatch]
pub trait NodeData {
    fn get_def(&self) -> Def;
    fn get_payload(&self) -> Option<ImSlice>;
}

pub trait Node<TChild>: NodeNav<TChild> + NodeData {}

impl<TChild, TNode: NodeData + NodeNav<TChild>> Node<TChild> for TNode {}

/// Information about the parent of a Node.
#[derive(Clone)]
pub struct ParentInfo<TNode> {
    pub node: TNode,
    pub label: Label,
}
