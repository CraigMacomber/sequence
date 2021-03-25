use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, Bencher, Criterion,
};
use rand::Rng;
use sequence::{
    basic,
    test_stuff::{chunked_tree, walk_all},
    Def, Label, NodeId,
};

use std::{cell::RefCell, rc::Rc};

fn big_basic_tree(size: usize) {
    let rng = Rc::new(RefCell::new(rand::thread_rng()));
    let new_node_id = || -> NodeId { NodeId(rng.borrow_mut().gen()) };
    let new_label = || -> Label { Label(rng.borrow_mut().gen()) };
    let new_def = || -> Def { Def(rng.borrow_mut().gen()) };

    let mut b = basic::BasicNode {
        def: new_def(),
        id: new_node_id(),
        payload: None,
        traits: std::collections::HashMap::new(),
    };

    let label = new_label();

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
}

fn walk_bench(b: &mut Bencher<WallTime>, size: usize, per_chunk: usize) {
    let (forest, id) = chunked_tree(size, per_chunk);
    let n = walk_all(forest.nav_from(id).unwrap());
    assert!(n >= size);
    assert!(n <= size * 2);
    b.iter(|| black_box(walk_all(forest.nav_from(id).unwrap())));
}

fn insert_bench(b: &mut Bencher<WallTime>, size: usize, per_chunk: usize) {
    b.iter(|| black_box(chunked_tree(size, per_chunk)));
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("big");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    group.significance_level(0.1).sample_size(10); //.measurement_time();

    // group.bench_function("walk 100 nodes", |b| walk_bench(b, 100, 0, 0));
    // group.bench_function("walk 1k nodes", |b| walk_bench(b, 1000, 0, 0));
    // group.bench_function("walk 10k nodes", |b| walk_bench(b, 10000, 0, 0));
    group.bench_function("walk 100k nodes", |b| walk_bench(b, 100000, 0));
    group.bench_function("walk 100k chunked", |b| walk_bench(b, 100000, 5));
    group.bench_function("walk 1m nodes", |b| walk_bench(b, 1000000, 0));
    group.bench_function("walk 1m small chunked", |b| walk_bench(b, 1000000, 5));
    group.bench_function("walk 1m big chunked", |b| walk_bench(b, 1000000, 5000));
    //group.bench_function("walk 10m nodes", |b| walk_bench(b, 10000000));
    // group.bench_function("insert 100 nodes", |b| insert_bench(b, 100, 0, 0));
    // group.bench_function("insert 1k nodes", |b| insert_bench(b, 1000, 0, 0));
    // group.bench_function("insert 10k nodes", |b| insert_bench(b, 10000, 0, 0));
    group.bench_function("insert 100k nodes", |b| insert_bench(b, 100000, 0));
    group.bench_function("insert 100k chunked", |b| insert_bench(b, 100000, 5));
    group.bench_function("insert 1m nodes", |b| insert_bench(b, 1000000, 0));
    group.bench_function("insert 1m small chunked", |b| insert_bench(b, 1000000, 5));
    group.bench_function("insert 1m big chunked", |b| insert_bench(b, 1000000, 5000));
    //group.bench_function("insert 10m nodes", |b| insert_bench(b, 10000000));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// sudo perf record --call-graph=dwarf ./benchmarks-dcce7430a8992982 --bench --profile-time 10
// sudo perf report --hierarchy -M intel
// https://rust-lang.github.io/packed_simd/perf-guide/prof/linux.html
// cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
// echo "performance" | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
