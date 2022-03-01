# Sequence

Experimental sequence and tree implementation.
Goals:

- a self describing tree of nodes with identities.
- efficient copy on write modifications.
- optimized for common cases for large sequences, allowing data like sequences of bytes to be real sequences of nodes with identity, but with only `O(log (n))` size overhead compared to storing them as a payload (no constant factor space or time over head per item).
- efficient lookup node data and parentage information (parent + some handle to where child is under parent) from identity of node.
- efficient membership testing of identity (checking that an identity is not in the tree is efficient)
- support lazy loading of subtrees.

Modifications (not currently implemented in this proof of concept) will be copy on write.
To support this the `im_rc` crate is used.
To further optimize this, the tree is indirected through a map by identity (see `Forest`).
This allows deep trees to be edited with cost `O(log(n)))` instead of requiring updating all the parents (worst case `O(n)`)

Handles general case, but allows optimizing for `chunks` of sequences using more efficient formats.
Cases supported so far:

- General case, but might have special cases below (see `indirect_dynamic`)
- Multiple nodes in a row in a sequence have the same schema, and sequential identities (see `chunk.rs`)

To allow chunks's internal nodes to be looked up efficiently, the indirection map is ordered, so looking up an identity it in can give the nearest preceding one, which will point to the chunk if its in one (chunks are identified by their first node's identity).
For efficient copy on write (and comparison), a b-tree is used for this (`im_rc:OrdMap`).

To allow efficient copy on write splitting and editing of values in a chunk (both `O(log(length of chunk))`),
the content of the chunk is stored in an im_rc::Vector.

## Lazy loading

Lazy loading (or more generally paging) is necessary for working with large trees.

Efficient membership testing, chunks, and data lookup from identity raises specific challenges for lazy loading.
Testing that an identity is not in the tree, including parts which have not been loaded, needs to be efficient.

There are two main ways to chunk the tree into lazy loadable chunks:

- by logical subtree
- by subtree in indirection b-tree

### Using the logical tree

Walking down the logical tree to lazy load content when looking up an identity should give good locality (chunks loaded will be useful for nearby parts of a tree and parents of loaded nodes are always loaded), so the `O(depth)` cost should be amortized pretty well.
This would rewire knowing which subtree to load, which could be done by storing a specialized (see below) bloom filter at every point where a separately loadable subtree is parented.

### Using indirection/identity b-tree

Alternatively loading directly from a virtualized (paged) indirection b-tree allows loading without loading a node from its id without loading the logical parents, avoiding the `O(depth)` time, bandwidth and memory, and instead is `O(log(n))`.
This seems better (avoids the need for the specialized bloom filter, tracking unloaded logical subtrees etc), but it amortizes poorly:
many lookups of nodes far apart in the identifier space will require pulling in different chunks.
The specialized bloom filter could still be used to accelerate negative queries (ones not in the tree), but it likely not work the complexity (positive queries are likely more common).

If poor amortization ends up being a problem there are some fixes that can be done:

- remapping the identities such that related nodes have nearby identities would fix it, and possibly allow storing shorter identities in the tree.
- accept the overhead for loading lots of b-tree nodes, but indirect the actual per node data (if non trivial in size) into chunks which tend to be grouped with nearby things in the tree (note that sequence `chunks` already do this for their special case)

## Chunk Specialized Bloom filter

Chunks define large ranges of identities.
A bloom filter which supports adding large ranges of identities efficiently can be constructed by using multiple bloom filters.
For example, for 128 bit identities, we could construct several bloom filters:

- one for individual identities, entered one at a time
- one for chunks of 256 identities (truncate identity's low byte before using this one): add range ~256 in side to this one (can add multiple entries as needed)
- one for chunks of 65536 identities
- optional more for even larger ranges if larger ranges are used.

This approach uses a constant number of bloom filters (ex: 3), and allows O(1) adding very large ranges (ex: millions of identities) in a bounded number of operations (set by which sizes of filters are used).

Ex:
Identities from 9832ed4c-56c1-44e3-9cb5-009a3acc71b8 to 9832ed4c-56c1-44e3-9cb5-009a3acd71b8 could be added to the one for ~256 sized chunks.
For readability, the identical high bits will be omitted, so they are: `...c71b8 to ...d00b8`.
That includes 36608 identities.
After truncating the low byte: `...c71 to ...d00` that leaves 143 identities.
These could be added to the second bloom filter.
Alternately it could be truncated again and `...c` and `...d` could be added to the third filter (this is more conservative: cheaper to add, more omits more false positives).

Filters could be added at any granularity of bits (not just every 8).

## Status

Early work in progress.
Nothing close to ready for use.

### Perf

Traverse a 1 million node tree in under a second, and insert those million nodes in 3 seconds (as independent copy on write inserts).

If the data is mostly big chunks, traversal is 0.2 seconds, and inserting the chunks (5k nodes each) is just 0.004 seconds.

All nodes can be looked up by their Id, but I'm not tracking parents yet (that won't change the algorithmic complexity, but will add significant a constant factor)

Size of ids mainly impacts memory use, and perf of inserting non-chunked data.

Traversing and inserting chunks is faster than basic nodes, even if the chunks are small (ex: 5 node color trees is > 5x faster to insert chunked and ~4x faster to walk).

## Ideas

Maybe use https://docs.rs/sanakirja/1.2.0/sanakirja/index.html (or specifically sanakirja-core if on the web) as its a B-Tree that supports paging and copy on write. Might work much better for server case (on disk cache, multi threaded etc).

When compressing ids, use context dependant short ids.

Maybe have internal state of an id compressor as part of the consensus document: only update it for sequenced ops.
Then ops can use ids that exist in there, full UUIDs, or an index relative to a known UUID.
Perhaps have ops that reserve a range so ops after it can specify a UUID+offset and get good auto clustering.

Alternatively, a smarter auto clustering solution than RangeTable.

Could use 32 bit short ids until they run out then switch to 64 (either support both mixed or convert all data).

Using data (instead for schema constants) for def_s or labels impacts compression scheme.

Could keep common schema in a buffer, remember state of this buffer for all revisions between minimum sequence and max sequenced revision (COW) (save this in snapshots), and use it to compress common types and ids on the wire (in ops).

Blobs within a snapshot can point to a blob which contains their schema data (or contain it directly.)
Just split-out and dedup all schema and Ids, then compress (ex: deflate) the whole thing. (Skip general compressed if storage layer already does it? Maybe keep compressed in memory for synchronous access)

Id compression and chunking are different subsets of extract then compress subset (mainly schema) pattern.

Chunks can be modified (ex: individual payloads changed) somewhat efficiently. Doing invalidation could be done with a diff on the data, and a byte -> id lookup.
Efficiently tracking observation of chunks is also an interesting problem. Maybe use an observations schema to prune diff? (hand over both snapshots when generating inval: use observation data to restrict compare to observed subset).
im_vec does not really what we want for chunk backing data (some sort of im_array would make more sense)

## Wasm

To build:

`wasm-pack build experimental-wasm`
