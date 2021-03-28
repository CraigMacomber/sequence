/*!
Experimental Copy On Write Tree in the style of [Fluid's experimental tree](https://github.com/microsoft/FluidFramework/tree/main/experimental/dds/tree#tree-abstraction).

This prototypes compressed sequences (See [chunk]),
identity compression (See [chunk] and [id_compress]),
a forest using [im_rc::OrdMap] which works with compressed sequences,
as well as a general architectural pattern for all of this with low coupling and a nice API in (See [nav]).

This design was done with virtualization (only loading a subset of the tree on demand) in mind, however none has been implemented yet.
The ability to load data on demand based on [NodeId], as well as efficiently look up parents is required.
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

use std::ops::{Add, Sub};

use util::ImSlice;

extern crate im_rc;
extern crate num_integer;

pub mod basic;
pub mod basic_indirect;
pub mod chunk;
pub mod forest;
pub mod id_compress;
pub mod indirect;
pub mod indirect_nav;
pub mod nav;
pub mod util;

pub mod test_stuff;

type IdBase = u128;

#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy)]
pub struct Def(pub IdBase);
#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy, Debug)]
pub struct Label(pub IdBase);

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

/// Navigation part of Node
pub trait NodeNav<TChild> {
    /// For iterating children within a trait.
    type TTraitChildren: Iterator<Item = TChild>;
    /// For iterating the set of trait labels for non-empty traits..
    type TLabels: Iterator<Item = Label>;

    // TODO: Performance: walking traits could be faster if this returned a reference to the trait not just the labels (saves a map lookup)
    fn get_traits(&self) -> Self::TLabels;
    fn get_trait(&self, label: Label) -> Self::TTraitChildren;
}

/// Tree Node.
/// Combines navigation with data (def and payload)
pub trait Node<TChild>: NodeNav<TChild> {
    fn get_def(&self) -> Def;
    fn get_payload(&self) -> Option<ImSlice>;
}

/// Id for a Node.
/// Some Nodes don't implement this because their Id can be instead be inferred from context (ex: key it is under in a map).
pub trait HasId {
    fn get_id(&self) -> NodeId;
}
