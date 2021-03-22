use crate::{
    basic_indirect::BasicNode,
    chunk::{Chunk, ChunkOffset},
};

// Chunk or BasicNode
type ChunkId = u128;

pub enum Nodes {
    Single(BasicNode),
    Chunk(Chunk),
}

// TODO: mutation APIs

pub struct Forest {
    // TODO: Both types of Nodes store their id, which is redundant with one in map.
    map: im_rc::OrdMap<ChunkId, Nodes>,
    // TODO parent information
}

pub enum NodeView<'a> {
    Single(&'a BasicNode),
    Chunk(ChunkOffset<'a>),
    // TODO: support undownloaded chunks blobs (find can return which blobs and at what offset the node is at)
    // TODO: support undownloaded subtrees that arn't chunks: find returns iterator of candidate trees using bloom filters
    // TODO: these types are write optimized. Consider supporting read/size optimized types (ex: using byte array instead of im's Vector)
}

impl Forest {
    pub fn find(&self, id: u128) -> Option<NodeView> {
        match self.map.get_prev(&id) {
            Some((k, v)) => match v {
                Nodes::Single(n) => Some(NodeView::Single(&n)),
                Nodes::Chunk(c) => c.lookup(id).map(|x| NodeView::Chunk(x)),
            },
            None => None,
        }
    }
}
