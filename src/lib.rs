use std::ops::{Add, Sub};

use util::ImSlice;

extern crate im_rc;
extern crate num_integer;

mod b_nav;
mod basic;
mod basic_indirect;
mod chunk;
mod dynamic;
mod forest;
mod indirect_dynamic;
mod nav;
mod util;

#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy)]
pub struct Def(u128);
#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy)]
pub struct Label(u128);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct NodeId(u128);

impl Add<usize> for NodeId {
    type Output = NodeId;

    fn add(self, rhs: usize) -> Self::Output {
        NodeId(self.0 + rhs as u128)
    }
}

impl Sub<NodeId> for NodeId {
    type Output = usize;

    fn sub(self, rhs: NodeId) -> Self::Output {
        (self.0 - rhs.0) as usize
    }
}

pub trait Node<TChild, Id> {
    type TTrait: Iterator<Item = TChild>;
    type TTraitIterator: IntoIterator<Item = Label>;

    fn get_id(&self) -> Id;
    fn get_def(&self) -> Def;
    fn get_payload(&self) -> Option<ImSlice>;

    fn get_traits(&self) -> Self::TTraitIterator;
    fn get_trait(&self, label: Label) -> Self::TTrait;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
