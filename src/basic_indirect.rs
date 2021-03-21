use std::collections::{hash_map::Keys, HashMap};

use crate::{util::CloneIterator, Node, Trait};

type ChildId = u128;
pub struct BasicNode {
    id: u128,
    def: u128,
    payload: Option<Vec<u8>>,
    traits: HashMap<u128, BasicTrait>,
}

pub struct BasicTrait {
    children: Vec<ChildId>,
}

impl<'a> Node<ChildId> for &'a BasicNode {
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

impl<'a> Trait<ChildId> for &'a BasicTrait {
    fn get_count(&self) -> usize {
        self.children.len()
    }
    fn get_child(&self, index: usize) -> ChildId {
        self.children[index]
    }
}
