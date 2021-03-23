use std::{
    collections::{hash_map::Keys, HashMap},
    iter::Cloned,
    slice,
};

use crate::{util::ImSlice, Def, Label, Node};

// Simple tree that owns generic tokens instead of its children.
// Due to issues with recursive types, this can't be used like `basic` to own its children.
// type BasicResucrsive<Id> = BasicNode<Id, BasicResucrsive<Id>>;

pub struct BasicNode<Id, Child> {
    id: Id,
    def: Def,
    payload: Option<im_rc::Vector<u8>>,
    traits: HashMap<Label, Vec<Child>>,
}

pub type BasicTrait<Child> = Vec<Child>;

impl<'a, Id: Copy, Child: Clone> Node<Child, Id> for &'a BasicNode<Id, Child> {
    type TTrait = Cloned<slice::Iter<'a, Child>>;
    type TTraitIterator = Cloned<Keys<'a, Label, Vec<Child>>>;

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
