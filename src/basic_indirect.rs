use std::{
    collections::{hash_map::Keys, HashMap},
    iter::Cloned,
};

use crate::{util::ImSlice, Def, Label, Node, Trait};

// Simple tree that owns generic tokens instead of its children.
// Due to issues with recursive types, this can't be used like `basic` to own its children.

pub struct BasicNode<Id, Child> {
    id: Id,
    def: Def,
    payload: Option<im_rc::Vector<u8>>,
    traits: HashMap<Label, BasicTrait<Child>>,
}

pub struct BasicTrait<Child> {
    pub children: Vec<Child>,
}

impl<'a, Id: Copy, Child: Copy> Node<Child, Id> for &'a BasicNode<Id, Child> {
    type TTrait = &'a BasicTrait<Child>;
    type TTraitIterator = Cloned<Keys<'a, Label, BasicTrait<Child>>>;

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

    fn get_trait(&self, label: Label) -> Option<Self::TTrait> {
        self.traits.get(&label)
    }
}

impl<'a, Child: Copy> Trait<Child> for &'a BasicTrait<Child> {
    fn get_count(&self) -> usize {
        self.children.len()
    }
    fn get_child(&self, index: usize) -> Child {
        self.children[index]
    }
}
