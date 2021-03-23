//! Simple tree that owns its children

use std::{
    collections::{hash_map::Keys, HashMap},
    iter::Cloned,
    slice,
};

use crate::{util::ImSlice, Def, Label, Node, NodeId};

pub struct BasicNode {
    id: NodeId,
    def: Def,
    payload: Option<im_rc::Vector<u8>>,
    traits: HashMap<Label, Vec<BasicNode>>, // TODO: Use hash map from im_rc
}

impl<'a> Node<&'a BasicNode, NodeId> for &'a BasicNode {
    type TTrait = slice::Iter<'a, BasicNode>;
    type TTraitIterator = Cloned<Keys<'a, Label, Vec<BasicNode>>>;

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

    fn get_trait(&self, label: Label) -> Self::TTrait {
        self.traits
            .get(&label)
            .map_or(EMPTY, |x| &x[..])
            .into_iter()
    }
}

const EMPTY: &[BasicNode] = &[];
