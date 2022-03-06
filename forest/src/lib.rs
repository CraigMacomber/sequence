/*!
Experimental Copy On Write Tree in the style of [Fluid's experimental tree](https://github.com/microsoft/FluidFramework/tree/main/experimental/dds/tree#tree-abstraction).

This prototypes a forest using [im_rc::OrdMap] which allows with compressed sequences via [uniform_chunk],
as well as a general architectural pattern for all of this with low coupling and a nice API in (See [nav]).

This design was done with virtualization (only loading a subset of the tree on demand) in mind, however none has been implemented yet.
The ability to load data on demand based on [node_id::NodeId], as well as efficiently look up parents is required.
The two main approaches for this would be to either virtualize the [forest]'s B Tree directly,
or to virtualize the logical tree, and load chunks of it into the Forest.

- Persisting the B tree:
    - Can be lazy loaded by Id easily, and can be chunked with efficient paging and copy on write updates to share data between snapshots.
    - We can factor out the schema information and actual payload data to make its size reasonable to load for large documents.
    - Partially loading the B-Tree be inefficient (in memory and bandwidth) when using small parts of the tree due to typical access patterns (like walking the logical tree) having poor locality.
        - Can we use an Id Compressor to occasionally remap UUIDs to short ID in a way to cause the b-tree to have good locality?
        - Compressed sequences, and other cluster based allocation schemes help, but might not be enough after lots of edits in large documents.
- Persist the logical tree in snapshots (Current approach in Fluid's experimental tree, but does not support virtualization):
    - When virtualized, hard to determine if a given Id exists, and if so load it's data.
        - Can track set of unloaded trees: use bloom filters to determine when they might be needed for Id lookup.
            - This has issues with sequence compression, but that can be solved with extra multiple bloom filters for truncated ids to emulate efficient batch insertion.
            - Higher code complexity, and possibly runtime cost (Update them on every edit?)
                - Maybe can use lazy update approach? (Dirty on change, recompute when saving)
                - Maybe use conservative updates (skip regenerating just to do deletes sometimes)
*/

use nav::Resolver;
use node_id::HasId;
use tree::ParentInfo;

extern crate derive_more;
extern crate im_rc;
extern crate num_integer;

mod chunk;
mod example_node;
mod forest;
mod indirect;
mod indirect_nav;
mod indirect_node;
mod nav;
mod node_id;
mod tree;
mod uniform_chunk;
mod util;

#[macro_use]
pub mod enum_node;
pub mod test_stuff;

/// A forest (collection of trees) that that can optionally compression sections using [uniform_chunk]s.
pub struct Forest {
    forest: indirect_nav::Forest,
    shapes: std::rc::Rc<ShapeLibrary>,
}

/// Unique identifier for a particular tree shape.
struct ShapeId(u128);

struct ShapeLibrary {
    // TODO: could use something like  weak_table::WeakValueHashMap if we don't want this to grow forever.
    map: std::collections::HashMap<ShapeId, uniform_chunk::RootChunkSchema>,
}

// Idea:
// Add getter for forest/resolver (generic type) to nav/tree (specifically indirect nav).
// Use this to add owning wrapper (that adds no fields) to allow editing by calling back into the forest (nav would have a mut forest in this case? Ref cell? COW ref cel?).
// Maybe do copy on write instead?

impl Forest {
    pub fn get_tree(&self, id: node_id::NodeId) -> Option<indirect::NodeView> {
        self.forest.find_node(id)
    }

    pub fn insert_or_replace_node(&mut self, node: impl tree::Node<node_id::NodeId> + HasId) {
        todo!()
        // Split chunks, rechunk etc.
    }

    pub fn delete_node(&mut self, id: node_id::NodeId) {
        todo!()
        // Split chunks, rechunk etc.
        // Return success or error has parent
    }

    pub fn get_parent(
        &self,
        id: node_id::NodeId,
    ) -> Option<Option<ParentInfo<indirect::NodeView>>> {
        let info = (&(self.forest)).get_parent(self.get_tree(id).as_ref()?);
        Some(info)
        // todo!("return better value for missing node vs root")
    }
}

/// Non-minimal functionality
impl Forest {
    /// Inserts a tree, allocating it's ids arbitrarily
    pub fn insert_tree(&mut self, tree: example_node::BasicNode) -> node_id::NodeId {
        todo!()
        // heuristically chunk
    }

    /// Insert a tree, keeping chunks as is.
    pub fn insert_chunked() {
        todo!()
        // dedup shapes
    }

    pub fn set_value(&mut self, id: node_id::NodeId, value: &[u8]) {
        todo!()
    }

    pub fn replace_node_chunked(&mut self, id: node_id::NodeId) {
        todo!()
    }

    pub fn update_chunk(
        &mut self,
        id: forest::ChunkId,
    ) -> im_rc::ordmap::Entry<forest::ChunkId, indirect_nav::NavChunk> {
        self.forest.entry(id)
    }

    pub fn delete_subtree(&mut self, id: node_id::NodeId) {
        todo!()
        // Return success or error has parent
        // Split chunks, rechunk etc.
    }
}
