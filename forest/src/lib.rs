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

extern crate derive_more;
extern crate enum_dispatch;
extern crate im_rc;
extern crate num_integer;

pub mod chunk;
pub mod example_node;
pub mod forest;
pub mod indirect;
pub mod indirect_nav;
pub mod indirect_node;
pub mod nav;
pub mod node_id;
pub mod tree;
pub mod uniform_chunk;
pub mod util;

pub mod test_stuff;
