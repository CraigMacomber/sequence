use util::ImSlice;

extern crate im_rc;
extern crate num_integer;

mod basic;
mod basic_indirect;
mod chunk;
mod dynamic;
mod indirect_dynamic;
mod util;

trait Node<TChild> {
    type TTrait: Trait<TChild>;
    type TTraitIterator: IntoIterator<Item = u128>;

    fn get_id(&self) -> u128;
    fn get_def(&self) -> u128;
    fn get_payload(&self) -> Option<ImSlice>;

    fn get_traits(&self) -> Self::TTraitIterator;
    fn get_trait(&self, label: u128) -> Option<Self::TTrait>;
}

trait Trait<TChild> {
    fn get_count(&self) -> usize;
    fn get_child(&self, index: usize) -> TChild;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
