//! Simple tree that owns its children.
//! This serves as an example of the simplest way to implement Node, and is actually used.

use std::{
    collections::{hash_map::Keys, HashMap},
    iter::Cloned,
    slice,
};

use crate::{util::ImSlice, Def, Label, Node, NodeId, NodeNav};

pub struct BasicNode {
    pub id: NodeId,
    pub def: Def,
    pub payload: Option<im_rc::Vector<u8>>,
    pub traits: HashMap<Label, Vec<BasicNode>>, // TODO: Use hash map from im_rc
}

impl<'a> NodeNav<&'a BasicNode> for &'a BasicNode {
    type TTraitChildren = slice::Iter<'a, BasicNode>;
    type TLabels = Cloned<Keys<'a, Label, Vec<BasicNode>>>;

    fn get_traits(&self) -> Self::TLabels {
        self.traits.keys().cloned()
    }

    fn get_trait(&self, label: Label) -> Self::TTraitChildren {
        self.traits.get(&label).map_or(EMPTY, |x| &x[..]).iter()
    }
}

impl Node for BasicNode {
    fn get_def(&self) -> Def {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.payload.as_ref().map(|p| p.focus())
    }
}

const EMPTY: &[BasicNode] = &[];
