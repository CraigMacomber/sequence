//! A `Chunk` of a Tree.

use crate::{IdOffset, NodeId};

/// A `Chunk` of a Tree.
/// Contains 0 or more nodes, all of which must have `NodeId` between (inclusive) some `first_id` and some `max_id`.
/// No chunk within the same forest can have a range of ids that overlaps with any other.
pub trait Chunk {
    /// The representation of Nodes in this Chunk.
    type View;

    /// gets an node with an id owned by this chunk
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<Self::View>;
}

pub trait OffsetBasedChunk {
    /// The representation of Nodes in this Chunk.
    type View;

    /// A chunk is allowed to be sparse within its range,
    /// however no ids within the range may be used elsewhere (it is considered to own them)
    fn max_offset(&self) -> IdOffset;

    fn get_from_offset(&self, first_id: NodeId, offset: IdOffset) -> Option<Self::View>;
}

impl<T: OffsetBasedChunk> Chunk for T {
    type View = T::View;
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<Self::View> {
        if id < first_id {
            None
        } else if first_id + self.max_offset() < id {
            self.get_from_offset(first_id, id - first_id)
        } else {
            None
        }
    }
}

pub trait Nodes: Chunk + Clone + PartialEq {}
