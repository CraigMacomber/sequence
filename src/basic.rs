use std::{
    collections::{hash_map::Keys, HashMap},
    iter::Cloned,
};

use crate::{util::ImSlice, Node, Trait};

pub struct BasicNode {
    id: u128,
    def: u128,
    payload: Option<im_rc::Vector<u8>>,
    traits: HashMap<u128, BasicTrait>, // TODO: Use hash map from im_rc
}

pub struct BasicTrait {
    children: Vec<BasicNode>, // TODO: Use vector from im_rc
}

impl<'a> Node<&'a BasicNode> for &'a BasicNode {
    type TTrait = &'a BasicTrait;
    type TTraitIterator = Cloned<Keys<'a, u128, BasicTrait>>;

    fn get_id(&self) -> u128 {
        self.id
    }

    fn get_def(&self) -> u128 {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        let o = &self.payload;
        match o {
            Some(p) => Some(p.into()),
            None => None,
        }
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: u128) -> Option<Self::TTrait> {
        self.traits.get(&label)
    }
}

impl<'a> Trait<&'a BasicNode> for &'a BasicTrait {
    fn get_count(&self) -> usize {
        self.children.len()
    }
    fn get_child(&self, index: usize) -> &'a BasicNode {
        &self.children[index]
    }
}
