use std::collections::{HashMap, hash_map::Keys};

use crate::{util::CloneIterator, Node, Trait};


pub struct BasicNode {
    id: u128,
    def: u128,
    payload: Option<Vec<u8>>,
    traits: HashMap<u128, BasicTrait>,
}

pub struct BasicTrait {
    children: Vec<BasicNode>,
}


impl<'a> Node for &'a BasicNode {
    type TTrait = &'a BasicTrait;
    type TTraitIterator = CloneIterator<Keys<'a, u128, BasicTrait>>;

    fn get_id(&self) -> u128 {
        self.id
    }

    fn get_def(&self) -> u128 {
        self.def
    }

    fn get_payload(&self) -> Option<&[u8]> {
        let o = &self.payload;
        match o {
            Some(p) => Some(p.as_slice()),
            None => None,
        }
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        CloneIterator {
            t: self.traits.keys(),
        }
    }

    fn get_trait(&self, label: u128) -> Option<Self::TTrait> {
        self.traits.get(&label)
    }
}

impl<'a> Trait for &'a BasicTrait {
    type TNode = &'a BasicNode;

    fn get_count(&self) -> usize {
        self.children.len()
    }
    fn get_child(&self, index: usize) -> Self::TNode {
        &self.children[index]
    }
}
