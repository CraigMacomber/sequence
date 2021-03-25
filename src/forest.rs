use std::cell::RefCell;

use crate::{util::ImHashMap, Label, NodeId, NodeNav};
use im_rc::ordmap::DiffItem;

// Chunk or BasicNode
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub struct ChunkId(pub NodeId);

pub trait Nodes: Clone {
    type View;

    /// A chunk is allowed to be sparse within its range,
    /// however no ids within the range may be used elsewhere (it is considered to own them)
    // fn max_offset(&self) -> IdOffset;

    /// gets an node with an id owned by this chunk
    fn get(&self, first_id: NodeId, id: NodeId) -> Option<Self::View>;

    // fn get(&self, first_id: NodeId, id: NodeId) -> Option<Self::View> {
    //     if id < first_id {
    //         None
    //     } else if first_id + self.max_offset() < id {
    //         self.get_from_offset(id, id - first_id)
    //     } else {
    //         None
    //     }
    // }
}

// TODO: mutation APIs

// Nodes added to forest must have non-overlapping ranges of Ids.
#[derive(Clone, Default)]
pub struct Forest<TNodes> {
    /// Up to date actual data of tree
    map: im_rc::OrdMap<ChunkId, TNodes>,
    /// Snapshot from last time parent_data was updated
    old_map: RefCell<im_rc::OrdMap<ChunkId, TNodes>>,
    /// Lazily updated parent data
    parent_data: RefCell<ImHashMap<ChunkId, ParentInfo<ChunkId>>>,
}

pub struct ForestParents<TNodes> {
    /// Snapshot from last time parent_data was updated
    // old_map: im_rc::OrdMap<ChunkId, TNodes>,
    map: im_rc::OrdMap<ChunkId, TNodes>,
    /// Lazily updated parent data
    pub parent_data: ImHashMap<ChunkId, ParentInfo<ChunkId>>,
}

impl<TNodes> Default for ForestParents<TNodes> {
    fn default() -> Self {
        ForestParents {
            map: im_rc::OrdMap::default(),
            parent_data: ImHashMap::default(),
        }
    }
}

impl<TNodes> Clone for ForestParents<TNodes> {
    fn clone(&self) -> Self {
        ForestParents {
            map: self.map.clone(),
            parent_data: self.parent_data.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ParentInfo<TNode> {
    pub node: TNode,
    pub label: Label,
}

impl<'a, TNodes: 'a> Forest<TNodes>
where
    &'a TNodes: Nodes,
{
    pub fn find_node(&'a self, id: NodeId) -> Option<<&'a TNodes as Nodes>::View> {
        match self.map.get_prev(&ChunkId(id)) {
            Some((chunk, v)) => v.get(chunk.0, id),
            None => None,
        }
    }
}

impl<'a, TNodes> Forest<TNodes> {
    pub fn new() -> Self {
        Forest {
            map: im_rc::OrdMap::default(),
            old_map: im_rc::OrdMap::default().into(),
            parent_data: ImHashMap::default().into(),
        }
    }

    pub fn find_nodes(&self, id: ChunkId) -> Option<&TNodes> {
        self.map.get(&id)
    }

    /// If there is an owning nodes for id, this returns it, but it may also return non owning nodes
    pub fn find_nodes_from_node(&'a self, id: NodeId) -> Option<(&'a ChunkId, &'a TNodes)> {
        self.map.get_prev(&ChunkId(id))
    }
}

fn update_forest_parents<'a, TNodes>(
    map: &'a im_rc::OrdMap<ChunkId, TNodes>,
    old_map: &'a im_rc::OrdMap<ChunkId, TNodes>,
    parent_data: &mut ImHashMap<ChunkId, ParentInfo<ChunkId>>,
) where
    TNodes: PartialEq<TNodes>,
    &'a TNodes: NodeNav<ChunkId>,
{
    for d in map.diff(&old_map) {
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
}

pub fn make_new_parent_info<'a, TNodes>(
    f: &'a Forest<TNodes>,
    p: &'a ForestParents<TNodes>,
) -> ForestParents<TNodes>
where
    TNodes: PartialEq<TNodes>,
    &'a TNodes: NodeNav<ChunkId>,
{
    let mut parent_data = p.parent_data.clone();
    update_forest_parents(&f.map, &p.map, &mut parent_data);

    ForestParents {
        map: f.map.clone(),
        parent_data,
    }
}

pub fn update_parent_info<'a, TNodes>(f: &'a Forest<TNodes>, p: &'a mut ForestParents<TNodes>)
where
    TNodes: PartialEq<TNodes>,
    &'a TNodes: NodeNav<ChunkId>,
{
    let result = make_new_parent_info(f, p);
    todo!()
    //*p = result;
}

impl<'a, TNodes: 'a> Forest<TNodes>
where
    TNodes: Clone,
{
    pub fn find_nodes_mut(&mut self, id: ChunkId) -> Option<&mut TNodes> {
        self.map.get_mut(&id)
    }

    /// Inserts a new chunk. May replace an existing one.
    pub fn insert(&mut self, id: ChunkId, value: TNodes) {
        self.map.insert(id, value);
    }
}
