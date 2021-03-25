use std::ops::{Add, Sub};

use util::ImSlice;

extern crate im_rc;
extern crate num_integer;

pub mod basic;
pub mod basic_indirect;
pub mod chunk;
pub mod forest;
pub mod id_compress;
pub mod indirect;
pub mod indirect_nav;
pub mod nav;
pub mod util;

pub mod test_stuff;

type IdBase = u128;

#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy)]
pub struct Def(pub IdBase);
#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy)]
pub struct Label(pub IdBase);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct NodeId(pub IdBase);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct IdOffset(pub u32);

impl Add<IdOffset> for NodeId {
    type Output = NodeId;

    fn add(self, rhs: IdOffset) -> Self::Output {
        NodeId(self.0 + rhs.0 as IdBase)
    }
}

impl Sub<NodeId> for NodeId {
    type Output = IdOffset;

    fn sub(self, rhs: NodeId) -> Self::Output {
        IdOffset((self.0 - rhs.0) as u32)
    }
}

/// Navigation part of Node
pub trait NodeNav<TChild> {
    type TTrait: IntoIterator<Item = TChild>;
    type TTraitIterator: IntoIterator<Item = Label>;

    // TODO: Performance: walking traits could be faster if this returned a reference to the trait not just the labels (saves a map lookup)
    fn get_traits(&self) -> Self::TTraitIterator;
    fn get_trait(&self, label: Label) -> Self::TTrait;
}

pub trait Node<TChild>: NodeNav<TChild> {
    fn get_def(&self) -> Def;
    fn get_payload(&self) -> Option<ImSlice>;
}

pub trait HasId {
    fn get_id(&self) -> NodeId;
}
