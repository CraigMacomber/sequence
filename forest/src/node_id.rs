use std::ops::{Add, Sub};

use enum_dispatch::enum_dispatch;

use crate::indirect::NodeView;

use crate::tree::IdBase;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct NodeId(pub IdBase);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct IdOffset(pub u32);

impl Add<IdOffset> for NodeId {
    type Output = NodeId;

    fn add(self, rhs: IdOffset) -> Self::Output {
        NodeId(self.0 + rhs.0 as IdBase)
    }
}

impl Sub<NodeId> for NodeId {
    type Output = IdOffset;

    fn sub(self, rhs: NodeId) -> Self::Output {
        IdOffset((self.0 - rhs.0) as u32)
    }
}

/// Id for a Node.
/// Some Nodes don't implement this because their Id can be instead be inferred from context (ex: key it is under in a map).
#[enum_dispatch]
pub trait HasId {
    fn get_id(&self) -> NodeId;
}
