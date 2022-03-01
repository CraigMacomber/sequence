use std::collections::HashMap;

extern crate forest;
extern crate uuid;

pub mod id_compress;

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
        for _n in 0..1000 {
            let _start = uuid::Uuid::new_v4().as_u128();
        }
    }
}

#[wasm_bindgen]
pub fn reserve_random1000() {
    for _n in 0..1000 {
        let _start = uuid::Uuid::new_v4().as_u128();
    }
}

#[wasm_bindgen]
pub fn add(a: usize, b: usize) -> usize {
    return a + b;
}

#[wasm_bindgen]
pub fn insert_bench(size: usize, per_chunk: usize, check_parents: bool) {
    let (t, _id) = forest::test_stuff::chunked_tree(size, per_chunk);
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
