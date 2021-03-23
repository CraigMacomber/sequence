use std::{
    collections::{hash_map::Keys, HashMap},
    iter::Cloned,
};

use crate::{util::ImSlice, Def, Label, Node, NodeId, Trait};

// Simple tree that owns its children

pub struct BasicNode {
    id: NodeId,
    def: Def,
    payload: Option<im_rc::Vector<u8>>,
    traits: HashMap<Label, BasicTrait>, // TODO: Use hash map from im_rc
}

pub struct BasicTrait {
    children: Vec<BasicNode>, // TODO: Use vector from im_rc
}

impl<'a> Node<&'a BasicNode, NodeId> for &'a BasicNode {
    type TTrait = &'a BasicTrait;
    type TTraitIterator = Cloned<Keys<'a, Label, BasicTrait>>;

    fn get_id(&self) -> NodeId {
        self.id
    }

    fn get_def(&self) -> Def {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.payload.as_ref().map(|p| p.focus())
    }

    fn get_traits(&self) -> Self::TTraitIterator {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Option<Self::TTrait> {
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
