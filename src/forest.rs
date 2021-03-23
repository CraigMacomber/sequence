use std::marker::PhantomData;

use crate::NodeId;

// Chunk or BasicNode
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct ChunkId(pub NodeId);

pub trait Nodes<View> {
    fn first_id(&self) -> NodeId;
    fn get(&self, id: NodeId) -> Option<View>;
}

// TODO: mutation APIs

// Nodes added to forest must have non-overlapping ranges of Ids.
pub struct Forest<TNodes, View>
where
    TNodes: Nodes<View>,
{
    // TODO: Nodes store their id (otherwise get can't be implemented), which is redundant with one in map.
    map: im_rc::OrdMap<ChunkId, TNodes>,
    phantom: PhantomData<dyn Nodes<View>>,
    // TODO parent information
}

impl<TNodes, View> Forest<TNodes, View>
where
    TNodes: Nodes<View>,
    ChunkId: Ord,
{
    pub fn find_nodes(&self, id: ChunkId) -> Option<&TNodes> {
        self.map.get(&id)
    }

    pub fn find_node(&self, id: NodeId) -> Option<View> {
        match self.map.get_prev(&ChunkId(id)) {
            Some((_, v)) => v.get(id),
            None => None,
        }
    }
}
