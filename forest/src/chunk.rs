//! A `Chunk` of a Tree.

use crate::{
    node_id::{HasId, NodeId},
    tree::{Node, NodeNav},
};

/// Id under which a Chunk is stored.
/// Must be equal to or precede all [NodeId]s present in the chunk.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub struct ChunkId(pub NodeId);

/// A `Chunk` of a Tree.
/// Contains 0 or more nodes, all of which must have `NodeId` between (inclusive) some `first_id` and some `max_id`.
/// No chunk within the same forest can have a range of ids that overlaps with any other.
///
/// NodeNav<ChunkId> is used to record chunk level parentage for parent lookup.
pub trait Chunk: Clone + PartialEq + NodeNav<ChunkId> {
    /// The representation of Nodes in this Chunk.
    type View: Node<Self::Child> + HasId;
    type Child;
    type Expander: Iterator<Item = Self::View>;

    /// gets an node with an id owned by this chunk
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<Self::View>;

    fn top_level_nodes(&self, first_id: NodeId) -> Self::Expander;
}

// /// A chunk that uses all ids in a range.
// /// TODO: this is currently unused, but it might be cleaner to refactor UniformChunk to use this?
// pub trait DenseChunk: Clone + PartialEq {
//     /// The representation of Nodes in this Chunk.
//     type View;

//     /// A chunk is allowed to be sparse within its range,
//     /// however no ids within the range may be used elsewhere (it is considered to own them)
//     fn max_offset(&self) -> IdOffset;

//     fn get_from_offset(&self, offset: IdOffset) -> Self::View;
// }

// impl<T: DenseChunk> Chunk for T {
//     type View = T::View;
//     fn get(&self, first_id: NodeId, id: NodeId) -> Option<Self::View> {
//         if id < first_id {
//             None
//         } else if first_id + self.max_offset() < id {
//             Some(self.get_from_offset(id - first_id))
//         } else {
//             None
//         }
//     }
// }
