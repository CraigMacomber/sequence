use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, Bencher, Criterion,
};
use rand::Rng;
use sequence::{
    basic,
    forest::ChunkId,
    test_stuff::{chunked_tree, walk_all, walk_direct_all},
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

fn walk_direct_bench(b: &mut Bencher<WallTime>, size: usize) {
    let (forest, id) = chunked_tree(size, 0);
    let n = walk_all(forest.nav_from(id).unwrap());
    assert!(n >= size);
    assert!(n <= size * 2);
    b.iter(|| black_box(walk_direct_all(&forest, ChunkId(id))));
}

fn insert_bench(b: &mut Bencher<WallTime>, size: usize, per_chunk: usize, check_parents: bool) {
    b.iter(|| {
        let (t, _id) = chunked_tree(size, per_chunk);
        if check_parents {
            t.get_parent_data();
        }
        black_box(t)
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("forest");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    group.significance_level(0.1).sample_size(10); //.measurement_time();

    for count in [1_000_000].iter().cloned() {
        group.bench_function(format!("{} node insert", count), |b| {
            insert_bench(b, count, 0, false)
        });
        group.bench_function(format!("{} node insert + update parents", count), |b| {
            insert_bench(b, count, 0, true)
        });
        group.bench_function(format!("{} node walk", count), |b| {
            walk_direct_bench(b, count)
        });
        group.bench_function(format!("{} node walk with nav", count), |b| {
            walk_bench(b, count, 0)
        });

        for chunk_size in [5, 1_000].iter().cloned() {
            group.bench_function(
                format!("{} node insert in chunks of {}", count, chunk_size),
                |b| insert_bench(b, count, chunk_size, false),
            );
            group.bench_function(
                format!(
                    "{} node insert in chunks of {} + update parents",
                    count, chunk_size
                ),
                |b| insert_bench(b, count, chunk_size, true),
            );
            group.bench_function(
                format!("{} node walk with nav over chunks of {}", count, chunk_size),
                |b| walk_bench(b, count, chunk_size),
            );
        }
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// sudo perf record --call-graph=dwarf ./benchmarks-dcce7430a8992982 --bench --profile-time 10
// sudo perf report --hierarchy -M intel
// https://rust-lang.github.io/packed_simd/perf-guide/prof/linux.html
// cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
// echo "performance" | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
