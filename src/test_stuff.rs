use crate::{basic, basic_indirect::BasicNode, forest, Def, Label, Node, NodeId};
use crate::{forest::ChunkId, indirect_nav::*};
use rand::Rng;
use std::{cell::RefCell, mem, rc::Rc};

pub fn big_tree(size: usize) -> (Forest, NodeId) {
    let mut forest = Forest::new();
    let rng = Rc::new(RefCell::new(rand::thread_rng()));
    let new_node_id = || -> NodeId { NodeId(rng.borrow_mut().gen()) };
    let new_chunk_id = || -> ChunkId { ChunkId(new_node_id()) };
    let newLabel = || -> Label { Label(rng.borrow_mut().gen()) };
    let new_def = || -> Def { Def(rng.borrow_mut().gen()) };

    let def = new_def();
    let root_id = new_node_id();
    let mut nodes = vec![root_id];
    let label = newLabel();

    forest.insert(
        ChunkId(root_id),
        NavChunk::Single(BasicNode {
            def,
            id: root_id,
            payload: None,
            traits: im_rc::HashMap::new(),
        }),
    );

    for i in 1..size {
        let id = new_node_id(); // NodeId(i as u128);
        forest.insert(
            ChunkId(id),
            NavChunk::Single(BasicNode {
                def,
                id,
                payload: None,
                traits: im_rc::HashMap::new(),
            }),
        );

        let parent_index = rng.borrow_mut().gen_range(0..nodes.len());
        let parent_id = nodes[parent_index];

        forest
            .map
            .entry(ChunkId(parent_id))
            .and_modify(|p| match p.as_ref() {
                NavChunk::Single(parent) => {
                    let mut new_parent = parent.clone();

                    new_parent
                        .traits
                        .entry(label)
                        .or_insert_with(|| vec![])
                        .push(ChunkId(id));

                    *p = Rc::new(NavChunk::Single(new_parent));
                }
                NavChunk::Chunk(_) => {
                    panic!()
                }
            });

        nodes.push(id);
    }

    // let an_id = forest.map.get_min().unwrap().0 .0;

    // let n = forest.find_node(an_id).unwrap();

    // let nav = forest.nav_from(an_id).unwrap();

    // let children: Vec<_> = nav.get_trait(Label(9)).collect();
    // assert!(children.len() == 0);

    // let n = forest.find_nodes(ChunkId(an_id)).unwrap();
    // let n = forest::Nodes::get(&n, an_id).unwrap();
    (forest, root_id)
}

pub fn walk_all<T: Node<T, NodeId>>(n: T) -> usize {
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
        let (forest, id) = big_tree(1000);
        let nav = forest.nav_from(id).unwrap();
        assert_eq!(walk_all(nav), size);
    }
}
