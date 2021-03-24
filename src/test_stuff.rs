use crate::{basic_indirect::BasicNode, Def, Label, Node, NodeId};
use crate::{forest::ChunkId, indirect_nav::*};
use rand::Rng;
use std::{cell::RefCell, rc::Rc};

pub fn big_tree(size: usize) -> (Forest, NodeId) {
    let mut forest = Forest::new();
    let rng = Rc::new(RefCell::new(rand::thread_rng()));
    let new_node_id = || -> NodeId { NodeId(rng.borrow_mut().gen()) };
    let new_label = || -> Label { Label(rng.borrow_mut().gen()) };
    let new_def = || -> Def { Def(rng.borrow_mut().gen()) };

    let def = new_def();
    let root_id = new_node_id();
    let mut nodes = vec![root_id];
    let label = new_label();

    forest.insert(
        ChunkId(root_id),
        NavChunk::Single(BasicNode {
            def,
            payload: None,
            traits: im_rc::HashMap::new(),
        }),
    );

    for _ in 1..size {
        let id = new_node_id(); // NodeId(i as u128);
        forest.insert(
            ChunkId(id),
            NavChunk::Single(BasicNode {
                def,
                payload: None,
                traits: im_rc::HashMap::new(),
            }),
        );

        let parent_index = rng.borrow_mut().gen_range(0..nodes.len());
        let parent_id = nodes[parent_index];

        let parent = forest.find_nodes_mut(ChunkId(parent_id)).unwrap();

        match parent {
            NavChunk::Single(basic) => {
                basic
                    .traits
                    .entry(label)
                    .or_insert_with(|| vec![])
                    .push(ChunkId(id));
            }
            NavChunk::Chunk(_) => {
                panic!();
            }
        };

        nodes.push(id);
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let size = 1000;
        let (forest, id) = big_tree(size);
        let nav = forest.nav_from(id).unwrap();
        assert_eq!(walk_all(nav), size);
    }

    #[test]
    fn big() {
        let size = 10000000;
        let (forest, id) = big_tree(size);
        let nav = forest.nav_from(id).unwrap();
        assert_eq!(walk_all(nav), size);
    }
}
