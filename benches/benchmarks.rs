use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use sequence::{
    basic,
    basic_indirect::BasicNode,
    forest,
    test_stuff::{big_tree, walk_all},
    Def, Label, Node, NodeId,
};
use sequence::{forest::ChunkId, indirect_nav::*};
use std::{cell::RefCell, mem, rc::Rc};

fn make_and_walk_tree(size: usize) {
    let (forest, id) = big_tree(1000);
    let nav = forest.nav_from(id).unwrap();
    walk_all(nav);
}

fn big_basic_tree(size: usize) {
    let mut forest = Forest::new();
    let rng = Rc::new(RefCell::new(rand::thread_rng()));
    let new_node_id = || -> NodeId { NodeId(rng.borrow_mut().gen()) };
    let new_chunk_id = || -> ChunkId { ChunkId(new_node_id()) };
    let newLabel = || -> Label { Label(rng.borrow_mut().gen()) };
    let new_def = || -> Def { Def(rng.borrow_mut().gen()) };

    let mut b = basic::BasicNode {
        def: new_def(),
        id: new_node_id(),
        payload: None,
        traits: std::collections::HashMap::new(),
    };

    let label = newLabel();

    b.traits.insert(
        label,
        (0..size)
            .into_iter()
            .map(|_| basic::BasicNode {
                def: new_def(),
                id: new_node_id(),
                payload: None,
                traits: std::collections::HashMap::new(),
            })
            .collect(),
    );

    // let an_id = forest.map.get_min().unwrap().0 .0;

    // let n = forest.find_node(an_id).unwrap();

    // let nav = forest.nav_from(an_id).unwrap();

    // let children: Vec<_> = nav.get_trait(Label(9)).collect();
    // assert!(children.len() == 0);

    // let n = forest.find_nodes(ChunkId(an_id)).unwrap();
    // let n = forest::Nodes::get(&n, an_id).unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let (forest, id) = big_tree(1000);
    let nav = forest.nav_from(id).unwrap();

    let mut group = c.benchmark_group("big");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    group.significance_level(0.2).sample_size(10);
    group.bench_function("insert 100 nodes", |b| b.iter(|| big_tree(100)));
    group.bench_function("insert 1k nodes", |b| b.iter(|| big_tree(1000)));
    group.bench_function("insert 10k nodes", |b| b.iter(|| big_tree(10000)));
    group.bench_function("insert 100k nodes", |b| b.iter(|| big_tree(100000)));
    group.bench_function("insert 1m nodes", |b| b.iter(|| big_tree(1000000)));
    group.bench_function("walk 1m nodes", |b| b.iter(|| walk_all(nav.clone())));
    // group.bench_function("insert 1m nodes basic", |b| {
    //     b.iter(|| big_basic_tree(1000000))
    // });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// sudo perf record --call-graph=dwarf ./benchmarks-dcce7430a8992982 --bench --profile-time 10
// sudo perf report --hierarchy -M intel
// https://rust-lang.github.io/packed_simd/perf-guide/prof/linux.html
