use std::rc::Rc;

use crate::NodeId;

// Chunk or BasicNode
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
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
            Some((chunk, v)) => v.as_ref().get(chunk.0, id),
            None => None,
        }
    }
}

impl<'a, TNodes: 'a> Forest<TNodes> {
    pub fn find_nodes(&self, id: ChunkId) -> Option<&TNodes> {
        self.map.get(&id).map(|b| b.as_ref())
    }

    pub fn find_nodes_mut(&mut self, id: ChunkId) -> Option<&mut Rc<TNodes>> {
        self.map.get_mut(&id)
    }

    /// Inserts a new chunk. May replace an existing one.
    pub fn insert(&mut self, id: ChunkId, value: TNodes) {
        self.map.insert(id, Rc::new(value));
    }

    pub fn new() -> Self {
        Forest {
            map: im_rc::OrdMap::new(),
        }
    }
}
