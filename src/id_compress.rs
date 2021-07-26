//! Tools for working with shorthands for UUIDs
//! The allocation scheme here is not stable, the short ids depend on the order they are requested.
//! The ShortIds are 64 bits (to make sure they don't run out), but an attempt is make to use the smaller ones first.
//! This is done so systems which handle smaller numbers more efficiently (ex: json serialization) or V8 can benefit.
//!
//! This is currently unused, and is just a experiment/example.

use std::{collections::HashMap, marker::PhantomData, ops::RangeInclusive};

pub trait IdCompressor<Long, Short> {
    fn shorten(&mut self, id: Long) -> Short;
    fn full(&self, id: Short) -> Long;
}

/// Id Compressor randomly distributed Ids.
pub trait TypedNumber {
    type N;
    fn as_number(&self) -> Self::N;
    fn from_number(n: Self::N) -> Self;
}

impl TypedNumber for u128 {
    type N = Self;

    fn as_number(&self) -> Self::N {
        *self
    }

    fn from_number(n: Self::N) -> Self {
        n
    }
}

impl TypedNumber for usize {
    type N = Self;

    fn as_number(&self) -> Self::N {
        *self
    }

    fn from_number(n: Self::N) -> Self {
        n
    }
}

impl TypedNumber for u32 {
    type N = Self;

    fn as_number(&self) -> Self::N {
        *self
    }

    fn from_number(n: Self::N) -> Self {
        n
    }
}

pub struct Table<Long, Short> {
    vec: Vec<Long>,
    map: HashMap<Long, Short, ahash::RandomState>, // TODO: maybe use ahash::RandomState?
}

impl<Long, Short> IdCompressor<Long, Short> for Table<Long, Short>
where
    Short: TypedNumber<N = usize> + Copy,
    Long: Copy + Eq + std::hash::Hash,
{
    fn shorten(&mut self, id: Long) -> Short {
        let map = &mut self.map;
        let vec = &mut self.vec;
        *map.entry(id).or_insert_with(|| {
            vec.push(id);
            TypedNumber::from_number(vec.len() - 1)
        })
    }

    fn full(&self, id: Short) -> Long {
        self.vec[id.as_number()]
    }
}

impl<Long, Short> Table<Long, Short> {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            map: HashMap::with_hasher(ahash::RandomState::new()),
        }
    }
}

/// Id Compressor optimized for numerically clustered ids.
/// Uses Table to shorten all but the low `SHIFT` bits, which are unchanged.
///
/// This reduces the number of entries table has to store if the ids are clustered
/// at the cost of going through the short id space faster (2^SHIFT faster in the worse case with no clustering).
///
/// In general this does NOT guarantee that runs of sequential of Long ids get sequential short ids.
/// See reserve_range if sequential ids are required.
pub struct RangeTable<Long, Short> {
    table: Table<u128, usize>,
    phantom: PhantomData<(Long, Short)>,
}

const SHIFT: usize = 10;
const MASK_LOW: usize = (1 << SHIFT) - 1;

impl<Long, Short> IdCompressor<Long, Short> for RangeTable<Long, Short>
where
    Short: TypedNumber<N = usize> + Copy,
    Long: Copy + Eq + std::hash::Hash + TypedNumber<N = u128>,
{
    fn shorten(&mut self, id: Long) -> Short {
        let inner_id = id.as_number() >> SHIFT;
        let inner = self.table.shorten(inner_id);

        TypedNumber::from_number((inner << SHIFT) | ((id.as_number() as usize) & MASK_LOW))
    }

    fn full(&self, id: Short) -> Long {
        let inner_id = id.as_number() >> SHIFT;
        let inner = self.table.full(inner_id);
        TypedNumber::from_number(((inner) << SHIFT) | ((id.as_number()) & MASK_LOW) as u128)
    }
}

impl<Long, Short> RangeTable<Long, Short> {
    pub fn new() -> Self {
        Self {
            table: Table::new(),
            phantom: PhantomData,
        }
    }
    /// For a range of Long Id_s which must either:
    /// Have never been seen before by this IdCompressor
    /// OR
    /// Is a subset of a range already reserved by this IdCompressor,
    /// get the corresponding range of Short ids.
    /// After calling this, the normal `shorten` and `full` functions can be used with these ids.
    ///
    /// Performance: Ranges are effectively rounded up and down to multiples of 2^SHIFT, so consider allocating ids accordingly.
    pub fn reserve_range(&mut self, ids: RangeInclusive<Long>) -> RangeInclusive<Short>
    where
        Short: TypedNumber<N = usize> + Copy,
        Long: Copy + Eq + std::hash::Hash + TypedNumber<N = u128>,
    {
        let inner_start = ids.start().as_number() >> SHIFT;
        let inner_end = ids.end().as_number() >> SHIFT;
        let mut previous = None;
        for i in inner_start..inner_end {
            let short = self.table.shorten(i);
            if let Some(prev) = previous {
                assert!(prev == short - 1);
            }
            previous = Some(short);
        }

        let r: RangeInclusive<Short> = self.shorten(*ids.start())..=self.shorten(*ids.end());

        debug_assert_eq!(
            (ids.end().as_number() - ids.start().as_number()),
            ((r.end().as_number() - r.start().as_number()) as u128)
        );
        r
    }
}
