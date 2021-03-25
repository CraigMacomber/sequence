//! Simple tree that owns generic tokens instead of its children.
//! Due to issues with recursive types, this can't be used like `basic` to own its children.
//! type BasicRecursive<Id> = BasicNode<Id, BasicRecursive<Id>>;

use std::{iter::Cloned, slice};

use crate::{
    util::{ImHashMap, ImSlice},
    Def, Label, Node, NodeNav,
};

#[derive(Clone, PartialEq)]
pub struct BasicNode<Child> {
    pub def: Def,
    // Payload is often not used, so indirect it to keep the size down.
    pub payload: Option<Box<im_rc::Vector<u8>>>,
    // TODO: use im::Vector here
    pub traits: ImHashMap<Label, Vec<Child>>,
}

impl<'a, Child: Clone> NodeNav<Child> for &'a BasicNode<Child> {
    type TTrait = Cloned<slice::Iter<'a, Child>>;
    type TTraitIterator = Cloned<im_rc::hashmap::Keys<'a, Label, Vec<Child>>>;

    fn get_traits(&self) -> Self::TTraitIterator {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        self.traits
            .get(&label)
            .map_or(BasicNode::<Child>::EMPTY.iter(), |x| x.iter())
            .cloned()
    }
}

impl<'a, Child: Clone> Node<Child> for &'a BasicNode<Child> {
    fn get_def(&self) -> Def {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        let o = &self.payload;
        match o {
            Some(p) => Some(p.focus()),
            None => None,
        }
    }
}

impl<'a, Child> BasicNode<Child> {
    const EMPTY: [Child; 0] = [];
}
