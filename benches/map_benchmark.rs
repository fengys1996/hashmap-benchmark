use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::Arc,
};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dashmap::DashMap;
use parking_lot::RwLock;

const PREFIX: &str = "halo_";

fn genrate_key(index: i32) -> String {
    format!("{}{}", PREFIX, index)
}

const WRITE_THREAD_NUM: i32 = 4;
const WRITE_CIRCLE_NUM: i32 = 2000;
const READ_THREAD_NUM: i32 = 16;
const READ_CIRCLE_NUM: i32 = 8000;

fn bench_dashmap() {
    // 1. init container
    let data_container = Arc::new(DashMap::with_capacity(300));

    // 2. start ten threads and execute
    let mut join_handlers = Vec::new();
    for _ in 0..WRITE_THREAD_NUM {
        let data_container = data_container.clone();
        let join_handler = std::thread::spawn(move || {
            for i in 0..WRITE_CIRCLE_NUM {
                data_container.insert(genrate_key(i), i);
            }
        });
        join_handlers.push(join_handler);
    }

    for _ in 0..READ_THREAD_NUM {
        let data_container = data_container.clone();
        let join_handler = std::thread::spawn(move || {
            for i in 0..READ_CIRCLE_NUM {
                let _ = data_container.get(&genrate_key(i));
            }
        });
        join_handlers.push(join_handler);
    }

    // 3. wait for ten threads to finish work
    for join_handler in join_handlers {
        join_handler.join().unwrap();
    }
}

fn bench_normal_map() {
    // 1. init container
    let mut data_container: Vec<RwLock<HashMap<String, i32>>> = Vec::with_capacity(300);
    for _ in 0..300 {
        data_container.push(Default::default());
    }
    let data_container = Arc::new(data_container);

    // 2. start ten threads and execute
    let mut join_handlers = Vec::new();
    for _ in 0..WRITE_THREAD_NUM {
        let mut hasher = DefaultHasher::default();
        let data_container = data_container.clone();
        let join_handler = std::thread::spawn(move || {
            for i in 0..WRITE_CIRCLE_NUM {
                let key = genrate_key(i);
                key.hash(&mut hasher);
                let index = hasher.finish() as usize % 300;
                let mut map = data_container[index].write();
                map.insert(genrate_key(i), i);
            }
        });
        join_handlers.push(join_handler);
    }

    for _ in 0..READ_THREAD_NUM {
        let mut hasher = DefaultHasher::default();
        let data_container = data_container.clone();
        let join_handler = std::thread::spawn(move || {
            for i in 0..READ_CIRCLE_NUM {
                let key = genrate_key(i);
                key.hash(&mut hasher);
                let index = hasher.finish() as usize % 300;
                let map = data_container[index].read();
                map.get(&genrate_key(i));
            }
        });
        join_handlers.push(join_handler);
    }

    // 3. wait for ten threads to finish work
    for join_handler in join_handlers {
        join_handler.join().unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_bench");
    for i in [20u64, 21u64].iter() {
        group.bench_function(BenchmarkId::new("dashmap", i), |b| {
            b.iter(|| bench_dashmap())
        });
        group.bench_function(BenchmarkId::new("normalmap", i), |b| {
            b.iter(|| bench_normal_map())
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
