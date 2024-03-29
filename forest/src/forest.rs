//! Collection of Trees
//!
//! Trees are stored as map from Id to Chunk, where a chunk is a collection of nodes within an id range (stored under the first id in the range).
//!
//! This is used by [crate::indirect_nav] to store and lookup [crate::indirect_nav::EnumChunk]s.

use std::cell::{Ref, RefCell};

use crate::{
    chunk::{Chunk, ChunkId},
    node_id::NodeId,
    tree::{NodeNav, ParentInfo},
    util::ImHashMap,
};
use im_rc::ordmap::DiffItem;

// Chunks added to forest must have non-overlapping ranges of Ids.
#[derive(Clone, Default)]
pub struct Forest<TChunk> {
    /// Up to date actual data of tree
    map: im_rc::OrdMap<ChunkId, TChunk>,
    /// Snapshot from last time parent_data was updated
    old_map: RefCell<im_rc::OrdMap<ChunkId, TChunk>>,
    /// Lazily updated parent data
    parent_data: RefCell<ImHashMap<ChunkId, ParentInfo<ChunkId>>>,
}

impl<TChunk> Forest<TChunk>
where
    TChunk: Clone + PartialEq<TChunk>,
    for<'a> &'a TChunk: Chunk,
{
    pub fn new() -> Self {
        Forest {
            map: im_rc::OrdMap::default(),
            old_map: im_rc::OrdMap::default().into(),
            parent_data: ImHashMap::default().into(),
        }
    }

    pub fn find_nodes(&self, id: ChunkId) -> Option<&TChunk> {
        self.map.get(&id)
    }

    /// If there is an owning nodes for id, this returns it, but it may also return non owning nodes
    pub fn find_nodes_from_node(&self, id: NodeId) -> Option<(&ChunkId, &TChunk)> {
        self.map.get_prev(&ChunkId(id))
    }

    pub fn find_nodes_mut(&mut self, id: ChunkId) -> Option<&mut TChunk> {
        self.map.get_mut(&id)
    }

    /// Inserts a new chunk. May replace an existing one.
    pub fn insert(&mut self, id: ChunkId, value: TChunk) {
        self.map.insert(id, value);
    }

    /// Inserts a new chunk. May replace an existing one.
    pub fn entry(&mut self, id: ChunkId) -> im_rc::ordmap::Entry<ChunkId, TChunk> {
        self.map.entry(id)
    }

    pub fn find_node(&self, id: NodeId) -> Option<<&TChunk as Chunk>::View> {
        match self.map.get_prev(&ChunkId(id)) {
            Some((chunk, v)) => v.get(chunk.0, id),
            None => None,
        }
    }

    pub fn get_parent_data(&self) -> Ref<ImHashMap<ChunkId, ParentInfo<ChunkId>>> {
        {
            let mut parent_data = self.parent_data.borrow_mut();
            for d in self.old_map.borrow().diff(&self.map) {
                match d {
                    DiffItem::Add(k, v) => {
                        for label in v.get_traits() {
                            for child in v.get_trait(label) {
                                parent_data.insert(child, ParentInfo { label, node: *k });
                            }
                        }
                    }
                    DiffItem::Update { old, new } => {
                        // TODO: Performance: could support efficient diff on Nodes, and do a much more optimal update here.
                        // For now, treat like remove then insert.
                        // TODO: quality: either do above, or deduplicate this.
                        let (_k, v) = old;
                        for label in v.get_traits() {
                            for child in v.get_trait(label) {
                                parent_data.remove(&child);
                            }
                        }
                        let (k, v) = new;
                        for label in v.get_traits() {
                            for child in v.get_trait(label) {
                                parent_data.insert(child, ParentInfo { label, node: *k });
                            }
                        }
                    }
                    DiffItem::Remove(_k, v) => {
                        for label in v.get_traits() {
                            for child in v.get_trait(label) {
                                parent_data.remove(&child);
                            }
                        }
                    }
                }
            }

            self.old_map.replace(self.map.clone());
        }
        self.parent_data.borrow()
    }

    pub fn get_parent_from_chunk_id(
        &self,
        id: ChunkId,
    ) -> Option<ParentInfo<<&TChunk as Chunk>::View>> {
        self.get_parent_data().get(&id).map(|x| ParentInfo {
            node: self.find_node(x.node.0).unwrap(),
            label: x.label,
        })
    }
}
