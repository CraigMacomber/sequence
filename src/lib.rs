use std::{
    collections::{
        hash_map::{self, Keys},
        HashMap,
    },
    ops::Deref,
    slice,
};

mod basic;
mod chunk;
mod util;
mod dynamic;

trait Node {
    type TTrait: Trait;
    type TTraitIterator: IntoIterator<Item = u128>;

    fn get_id(&self) -> u128;
    fn get_def(&self) -> u128;
    fn get_payload(&self) -> Option<&[u8]>;

    fn get_traits(&self) -> Self::TTraitIterator;
    fn get_trait(&self, label: u128) -> Option<Self::TTrait>;
}

enum Containment {
    Yes,
    No,
    Maybe,
}

trait Trait {
    type TNode;

    fn get_count(&self) -> usize;
    fn get_child(&self, index: usize) -> Self::TNode;
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
