//! Utilities for viewing trees of mixed [crate::basic_indirect::BasicNode] and [crate::chunk::UniformChunk].
//! where the children of the basic nodes are [ChunkId]s which correspond to either a chunk, or a BasicNode.
//!
//! The [crate::Node] implemented here is abstracts this as children are which are either Nodes, or ChunkIds.
//! When traversing within a chunk, no indirection is involved, looking up an Id is only required when traversing a BasicNode.
//!
//! Abstracting away this difference
//! (and dealing with the fact that a trait may contain a mix of chunks and basic nodes, and the chunks might contain multiple top level nodes)
//! is done by [crate::indirect_nav] which wraps this node in a Node implementation up with a forest using [crate::nav::Nav].

use crate::{forest::ChunkId, indirect_node::IndirectNode, uniform_chunk::UniformChunkNode};

// TODO: support undownloaded chunks blobs (find can return which blobs and at what offset the node is at)
// TODO: support undownloaded subtrees that arn't chunks: find returns iterator of candidate trees using bloom filters
// TODO: these types are write optimized. Consider supporting read/size optimized types (ex: using byte array instead of im's Vector)
// TODO: maybe chunks referencing external subtrees (so they can have child references like payloads)

crate::enum_node::fromMembers![
    (IndirectNode, ChunkId),
    (UniformChunkNode, UniformChunkNode<'a>)
];
