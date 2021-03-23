use std::marker::PhantomData;

use crate::NodeId;

// Chunk or BasicNode
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct ChunkId(pub NodeId);

pub trait Nodes<View>: Clone {
    fn first_id(&self) -> NodeId;

    // gets an node with an id owned by this chunk
    fn get(&self, id: NodeId) -> Option<View>;
}

// TODO: mutation APIs

// Nodes added to forest must have non-overlapping ranges of Ids.
#[derive(Clone)]
pub struct Forest<TNodes, View> {
    // TODO: Nodes store their id (otherwise get can't be implemented), which is redundant with one in map.
    pub map: im_rc::OrdMap<ChunkId, TNodes>,
    // TODO: is this right?
    phantom: PhantomData<fn(TNodes) -> View>,
    // TODO parent information
}

impl<'a, TNodes: 'a, View: 'a> Forest<TNodes, View>
where
    &'a TNodes: Nodes<View>,
{
    pub fn find_nodes(&self, id: ChunkId) -> Option<&TNodes> {
        self.map.get(&id)
    }

    pub fn find_node(&'a self, id: NodeId) -> Option<View> {
        match self.map.get_prev(&ChunkId(id)) {
            Some((_, v)) => v.get(id),
            None => None,
        }
    }

    pub fn new() -> Self {
        Forest {
            map: im_rc::OrdMap::new(),
            phantom: PhantomData,
        }
    }
}
