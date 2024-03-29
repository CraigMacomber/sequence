use crate::{
    chunk::ChunkId,
    indirect::enum_chunk,
    indirect_nav::*,
    indirect_node::IndirectChunk,
    node_id::{IdOffset, NodeId},
    tree::{Def, Label, Node, NodeNav},
    uniform_chunk::{ChunkSchema, OffsetSchema, RootChunkSchema, UniformChunk},
};
use rand::Rng;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub const PER_CHUNK_ITEM: usize = 5;

pub fn chunked_tree(size: usize, per_chunk: usize) -> (Forest, NodeId) {
    let (chunks, chunk_size) = if per_chunk == 0 {
        (0, 0)
    } else {
        let chunk_size = per_chunk / PER_CHUNK_ITEM;
        let chunks = size / (chunk_size * PER_CHUNK_ITEM);
        (chunks, chunk_size)
    };

    // Make sure we have at least enough nodes to make reasonable palaces to parent chunks under, and enough to really have `size` nodes.
    let basic_nodes = usize::max(
        1 + chunks / 10,
        size - (chunks * PER_CHUNK_ITEM * chunk_size),
    );
    big_tree(basic_nodes, chunks, chunk_size)
}

pub fn big_tree(size: usize, chunks: usize, chunk_size: usize) -> (Forest, NodeId) {
    let mut forest = Forest::new();

    let rng = RefCell::new(rand::thread_rng());

    // let id = RefCell::new(0);
    // let new_node_id = || {
    //     let mut id = id.borrow_mut();
    //     *id = *id + 1;
    //     NodeId(*id)
    // };
    let new_node_id = || NodeId(rng.borrow_mut().gen());

    let new_label = || -> Label { Label(rng.borrow_mut().gen()) };
    let new_def = || -> Def { Def(rng.borrow_mut().gen()) };

    let def = new_def();
    let root_id = new_node_id();
    let mut nodes = vec![root_id];
    let label = new_label();

    forest.insert(
        ChunkId(root_id),
        IndirectChunk {
            def,
            payload: None,
            traits: im_rc::HashMap::default(),
        }
        .into(),
    );

    for _ in 1..size {
        let id = new_node_id();
        forest.insert(
            ChunkId(id),
            IndirectChunk {
                def,
                payload: None, //Some(im_rc::Vector::from_iter([1u8].iter().cloned()).into()),
                traits: im_rc::HashMap::default(),
            }
            .into(),
        );

        let parent_index = rng.borrow_mut().gen_range(0..nodes.len());
        let parent_id = nodes[parent_index];

        let parent = forest.find_nodes_mut(ChunkId(parent_id)).unwrap();

        match parent {
            enum_chunk::Chunk::Indirect(basic) => {
                basic
                    .traits
                    .entry(label)
                    .or_insert_with(Vec::new)
                    .push(ChunkId(id));
            }
            _ => panic!(),
        };

        nodes.push(id);
    }

    if chunks > 0 {
        // color channel schema
        let sub_schema = ChunkSchema {
            def: new_def(),
            node_count: 1,
            bytes_per_node: 1,
            id_stride: 1,
            payload_size: Some(1),
            traits: HashMap::default(),
        };

        // Color schema (rgba)
        let schema = ChunkSchema {
            def: new_def(),
            node_count: chunk_size as u32,
            bytes_per_node: 4,
            id_stride: 5,
            payload_size: None,
            traits: vec![
                (
                    new_label(),
                    OffsetSchema {
                        id_offset: IdOffset(1),
                        byte_offset: 0,
                        schema: sub_schema.clone(),
                    },
                ),
                (
                    new_label(),
                    OffsetSchema {
                        id_offset: IdOffset(2),
                        byte_offset: 1,
                        schema: sub_schema.clone(),
                    },
                ),
                (
                    new_label(),
                    OffsetSchema {
                        id_offset: IdOffset(3),
                        byte_offset: 2,
                        schema: sub_schema.clone(),
                    },
                ),
                (
                    new_label(),
                    OffsetSchema {
                        id_offset: IdOffset(4),
                        byte_offset: 3,
                        schema: sub_schema,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        let chunk_schema = Rc::new(RootChunkSchema::new(schema));

        for _ in 0..chunks {
            let id = new_node_id();
            let data: im_rc::Vector<u8> = std::iter::repeat(&[1u8, 2, 3, 4])
                .take(chunk_size)
                .flat_map(|x| x.iter())
                .cloned()
                .collect();
            debug_assert_eq!(data.len(), chunk_size * 4);
            forest.insert(
                ChunkId(id),
                enum_chunk::Chunk::Uniform(UniformChunk {
                    schema: chunk_schema.clone(),
                    data: data.into(),
                }),
            );

            let parent_index = rng.borrow_mut().gen_range(0..nodes.len());
            let parent_id = nodes[parent_index];

            let parent = forest.find_nodes_mut(ChunkId(parent_id)).unwrap();

            match parent {
                enum_chunk::Chunk::Indirect(basic) => {
                    basic
                        .traits
                        .entry(label)
                        .or_insert_with(Vec::new)
                        .push(ChunkId(id));
                }
                _ => panic!(),
            };
        }
    }

    (forest, root_id)
}

pub fn simple_tree() -> (Forest, NodeId) {
    let mut forest = Forest::new();
    let _id = RefCell::new(0);
    let rng = RefCell::new(rand::thread_rng());
    let new_node_id = || NodeId(rng.borrow_mut().gen());

    let new_label = || -> Label { Label(rng.borrow_mut().gen()) };
    let new_def = || -> Def { Def(rng.borrow_mut().gen()) };

    let def = new_def();
    let root_id = new_node_id();
    let nodes = vec![root_id];
    let label = new_label();

    forest.insert(
        ChunkId(root_id),
        IndirectChunk {
            def,
            payload: None,
            traits: im_rc::HashMap::default(),
        }
        .into(),
    );

    // color channel schema
    let sub_schema = ChunkSchema {
        def: new_def(),
        node_count: 1,
        bytes_per_node: 1,
        id_stride: 1,
        payload_size: Some(1),
        traits: HashMap::default(),
    };

    // Color schema (rgba)
    let schema = ChunkSchema {
        def: new_def(),
        node_count: 2,
        bytes_per_node: 1,
        id_stride: 2,
        payload_size: None,
        traits: vec![(
            new_label(),
            OffsetSchema {
                id_offset: IdOffset(1),
                byte_offset: 0,
                schema: sub_schema,
            },
        )]
        .into_iter()
        .collect(),
    };

    let chunk_schema = Rc::new(RootChunkSchema::new(schema));

    for _ in 0..1 {
        let id = new_node_id();
        let data: im_rc::Vector<u8> = std::iter::repeat(&[1u8, 2])
            .take(1)
            .flat_map(|x| x.iter())
            .cloned()
            .collect();
        forest.insert(
            ChunkId(id),
            enum_chunk::Chunk::Uniform(UniformChunk {
                schema: chunk_schema.clone(),
                data: data.into(),
            }),
        );

        let parent_index = rng.borrow_mut().gen_range(0..nodes.len());
        let parent_id = nodes[parent_index];

        let parent = forest.find_nodes_mut(ChunkId(parent_id)).unwrap();

        match parent {
            enum_chunk::Chunk::Indirect(basic) => {
                basic
                    .traits
                    .entry(label)
                    .or_insert_with(Vec::new)
                    .push(ChunkId(id));
            }
            _ => panic!(),
        };
    }

    (forest, root_id)
}

pub fn walk_all<T: Node<T>>(n: T) -> usize {
    let mut count = 1;
    for t in n.get_traits() {
        for c in n.get_trait(t) {
            count += walk_all(c);
        }
    }
    count
}

pub fn walk_direct_all(forest: &Forest, id: ChunkId) -> usize {
    let mut count = 1;
    let n = forest.find_nodes(id).unwrap();
    for t in n.get_traits() {
        for c in n.get_trait(t) {
            count += walk_direct_all(forest, c);
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indirect::enum_chunk;
    use crate::nav::{Nav, WithParent};
    use crate::node_id::HasId;
    use crate::tree::NodeNav;

    #[test]
    fn basic_nodes() {
        let size = 1000;
        let (forest, id) = big_tree(size, 0, 1000);
        let nav = forest.nav_from(id).unwrap();
        assert_eq!(walk_all(nav), size);
    }

    #[test]
    fn parents() {
        let size = 100;
        let (forest, id) = big_tree(size, 0, 100);
        let nav = forest.nav_from(id).unwrap();
        check_parents(nav);
    }

    #[test]
    fn parents_with_chunks() {
        let size = 100;
        let (forest, id) = big_tree(size, 5, 100);
        let nav = forest.nav_from(id).unwrap();
        check_parents(nav);
    }

    #[test]
    fn parents_with_chunks_small() {
        let size = 1;
        let (forest, id) = big_tree(size, 1, 2);
        let nav = forest.nav_from(id).unwrap();
        check_parents(nav);
    }

    #[test]
    fn parents_with_chunks_small2() {
        let (forest, id) = simple_tree();
        let nav = forest.nav_from(id).unwrap();
        check_parents(nav);
    }

    #[test]
    fn parents_with_chunk() {
        let mut forest = Forest::new();
        let id = RefCell::new(0);
        let rng = RefCell::new(rand::thread_rng());

        let new_node_id = || {
            let mut id = id.borrow_mut();
            *id = *id + 1;
            NodeId(*id)
        };
        //let new_node_id = || NodeId(rng.borrow_mut().gen());

        let new_label = || -> Label { Label(rng.borrow_mut().gen()) };
        let new_def = || -> Def { Def(rng.borrow_mut().gen()) };

        // color channel schema
        let sub_schema = ChunkSchema {
            def: new_def(),
            node_count: 1,
            bytes_per_node: 1,
            id_stride: 1,
            payload_size: Some(1),
            traits: HashMap::default(),
        };

        // Color schema (rgba)
        let schema = ChunkSchema {
            def: new_def(),
            node_count: 2,
            bytes_per_node: 2,
            id_stride: 5,
            payload_size: None,
            traits: vec![
                (
                    new_label(),
                    OffsetSchema {
                        id_offset: IdOffset(1),
                        byte_offset: 0,
                        schema: sub_schema.clone(),
                    },
                ),
                (
                    new_label(),
                    OffsetSchema {
                        id_offset: IdOffset(2),
                        byte_offset: 1,
                        schema: sub_schema.clone(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        let chunk_schema = Rc::new(RootChunkSchema::new(schema));

        let id = new_node_id();
        let data: im_rc::Vector<u8> = [1u8, 2, 3, 4].iter().cloned().collect();
        forest.insert(
            ChunkId(id),
            enum_chunk::Chunk::Uniform(UniformChunk {
                schema: chunk_schema.clone(),
                data: data.into(),
            }),
        );

        let nav = forest.nav_from(id).unwrap();
        check_parents(nav);
    }

    pub fn check_parents(n: impl WithParent + Node + HasId) {
        for t in n.get_traits() {
            for c in n.get_trait(t) {
                let p = c.parent().unwrap();
                assert_eq!(p.label, t);
                // println!("{}  {}  {}", p.node.get_id().0, n.get_id().0, c.get_id().0);
                assert_eq!(p.node.get_id(), n.get_id());
                check_parents(c);
            }
        }
    }

    #[test]
    fn with_chunks() {
        const PER_CHUNK_ITEM: usize = 5;
        let size = 10;
        let chunks = 10;
        let chunk_size = 100;
        let (forest, id) = big_tree(size, chunks, chunk_size);
        let nav = forest.nav_from(id).unwrap();
        let n = walk_all(nav);
        assert_eq!(n, size + chunks * chunk_size * PER_CHUNK_ITEM);
    }

    #[test]
    fn with_chunks2() {
        let size = 100;
        let chunks = 200;
        let chunk_size = 1000;
        let (forest, id) = big_tree(size, chunks, chunk_size);
        let nav = forest.nav_from(id).unwrap();
        let n = walk_all(nav);
        assert_eq!(n, size + chunks * chunk_size * PER_CHUNK_ITEM);
    }

    // #[test]
    // fn big() {
    //     let size = 100000;
    //     let (forest, id) = big_tree(size, 0, 1000);
    //     let nav = forest.nav_from(id).unwrap();
    //     assert_eq!(walk_all(nav), size);
    // }

    #[test]
    fn print_sizes() {
        println!(
            "Chunk:{} BasicNode:{} EnumChunk:{}, ahash ImMap:{}, Default ImMap:{}, stdMap:{}",
            std::mem::size_of::<UniformChunk>(),
            std::mem::size_of::<IndirectChunk>(),
            std::mem::size_of::<enum_chunk::Chunk>(),
            std::mem::size_of::<im_rc::HashMap<Label, Vec<ChunkId>, ahash::RandomState>>(),
            std::mem::size_of::<im_rc::HashMap<Label, Vec<ChunkId>>>(),
            std::mem::size_of::<std::collections::HashMap<Label, Vec<ChunkId>>>(),
        );
        // panic!();
    }
}
