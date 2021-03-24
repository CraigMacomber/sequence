use std::rc::Rc;

use crate::NodeId;

// Chunk or BasicNode
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct ChunkId(pub NodeId);

pub trait Nodes: Clone {
    type View;
    // fn first_id(&self) -> NodeId;

    // gets an node with an id owned by this chunk
    fn get(&self, id: NodeId) -> Option<Self::View>;
}

// TODO: mutation APIs

// Nodes added to forest must have non-overlapping ranges of Ids.
#[derive(Clone)]
pub struct Forest<TNodes> {
    // TODO: Nodes store their id (otherwise get can't be implemented), which is redundant with one in map.
    pub map: im_rc::OrdMap<ChunkId, Rc<TNodes>>,
}

impl<'a, TNodes: 'a> Forest<TNodes>
where
    &'a TNodes: Nodes,
{
    pub fn find_node(&'a self, id: NodeId) -> Option<<&'a TNodes as Nodes>::View> {
        match self.map.get_prev(&ChunkId(id)) {
            Some((_, v)) => v.as_ref().get(id),
            None => None,
        }
    }
}

impl<'a, TNodes: 'a> Forest<TNodes> {
    pub fn find_nodes(&self, id: ChunkId) -> Option<&TNodes> {
        self.map.get(&id).map(|b| b.as_ref())
    }

    pub fn insert(&mut self, id: ChunkId, value: TNodes) {
        self.map.insert(id, Rc::new(value));
    }

    pub fn new() -> Self {
        Forest {
            map: im_rc::OrdMap::new(),
        }
    }
}
