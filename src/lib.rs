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

use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

use indirect::NodeView;

use util::ImSlice;

extern crate derive_more;
extern crate enum_dispatch;
extern crate im_rc;
extern crate num_integer;
extern crate uuid;

use enum_dispatch::enum_dispatch;

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
#[enum_dispatch]
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
#[enum_dispatch]
pub trait Node {
    fn get_def(&self) -> Def;
    fn get_payload(&self) -> Option<ImSlice>;
}

/// Id for a Node.
/// Some Nodes don't implement this because their Id can be instead be inferred from context (ex: key it is under in a map).
#[enum_dispatch]
pub trait HasId {
    fn get_id(&self) -> NodeId;
}

use id_compress::IdCompressor;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn setup() {
    #[cfg(debug_assertions)]
    {
        set_panic_hook();
    }
}

#[wasm_bindgen]
pub struct UuidShortener {
    table: id_compress::RangeTable<u128, usize>,
}

#[wasm_bindgen]
impl UuidShortener {
    pub fn new() -> Self {
        Self {
            table: id_compress::RangeTable::new(),
        }
    }

    #[wasm_bindgen]
    pub fn shorten_array(&mut self, base: &[u8]) -> usize {
        // This pulls in Result and Error, totaling ~10 kb.
        // let array = base.try_into().unwrap();
        // Avoid result and error by manually doing conversion:
        let array = [
            base[0], base[1], base[2], base[3], base[4], base[5], base[6], base[7], base[8],
            base[9], base[10], base[11], base[12], base[13], base[14], base[15],
        ];
        let base_id = u128::from_be_bytes(array);
        let short = self.table.shorten(base_id);
        return short;
    }

    #[wasm_bindgen]
    pub fn shorten_string(&mut self, base: String) -> usize {
        let base_id = uuid::Uuid::parse_str(&base).unwrap().as_u128();
        let short = self.table.shorten(base_id);
        return short;
    }

    #[wasm_bindgen]
    pub fn full(&mut self, id: usize) -> String {
        uuid::Uuid::from_u128(self.table.full(id))
            .to_hyphenated()
            .to_string()
    }

    #[wasm_bindgen]
    pub fn reserve(&mut self, base: usize, offset: usize, count: usize) -> usize {
        let start = self.table.full(base) + (offset as u128);
        *self
            .table
            .reserve_range(start..=(start + (count as u128)))
            .start()
    }

    #[wasm_bindgen]
    pub fn reserve_random(&mut self, count: usize) -> usize {
        let start = uuid::Uuid::new_v4().as_u128();
        *self
            .table
            .reserve_range(start..=(start + ((count - 1) as u128)))
            .start()
    }

    #[wasm_bindgen]
    pub fn reserve_random1000(&mut self) {
        for n in 0..1000 {
            let start = uuid::Uuid::new_v4().as_u128();
        }
    }
}

#[wasm_bindgen]
pub fn reserve_random1000() {
    for n in 0..1000 {
        let start = uuid::Uuid::new_v4().as_u128();
    }
}

#[wasm_bindgen]
pub fn add(a: usize, b: usize) -> usize {
    return a + b;
}

#[wasm_bindgen]
pub fn insert_bench(size: usize, per_chunk: usize, check_parents: bool) {
    let (t, _id) = test_stuff::chunked_tree(size, per_chunk);
    if check_parents {
        t.get_parent_data();
    }
}

struct FullId(u128);
typed_number_for_struct!(FullId, u128);

struct ShortId(usize);
typed_number_for_struct!(ShortId, usize);

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct TestMap {
    map: HashMap<u128, u32, ahash::RandomState>,
}

#[wasm_bindgen]
impl TestMap {
    #[wasm_bindgen]
    pub fn new() -> Self {
        Self {
            map: HashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    #[wasm_bindgen]
    pub fn add(&mut self, base: String) {
        let base_id = uuid::Uuid::parse_str(&base).unwrap().as_u128();
        let n = self.map.len() as u32;
        self.map.insert(base_id, n);
    }
}
