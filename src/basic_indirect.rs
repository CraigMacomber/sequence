//! Simple tree that owns generic tokens instead of its children.
//! Due to issues with recursive types, this can't be used like `basic` to own its children.
//! type BasicRecursive<Id> = BasicNode<Id, BasicRecursive<Id>>;

use std::{iter::Cloned, slice};

use crate::{util::ImSlice, Def, Label, Node};

#[derive(Clone)]
pub struct BasicNode<Id, Child> {
    pub id: Id,
    pub def: Def,
    pub payload: Option<im_rc::Vector<u8>>,
    // TODO: use im::Vector here
    pub traits: im_rc::HashMap<Label, Vec<Child>>,
}

impl<'a, Id: Copy, Child: Clone> Node<Child, Id> for &'a BasicNode<Id, Child> {
    type TTrait = Cloned<slice::Iter<'a, Child>>;
    type TTraitIterator = Cloned<im_rc::hashmap::Keys<'a, Label, Vec<Child>>>;

    fn get_id(&self) -> Id {
        self.id
    }

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

    fn get_traits(&self) -> Self::TTraitIterator {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTrait {
        self.traits
            .get(&label)
            .map_or(BasicNode::<Id, Child>::EMPTY.iter(), |x| x.iter())
            .cloned()
    }
}

impl<'a, Id, Child> BasicNode<Id, Child> {
    const EMPTY: [Child; 0] = [];
}
