use fractional_cascading::FractionalCascading;

use rand::SeedableRng;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use rand::seq::SliceRandom;

fn find_with_binary_search<T: Ord + Copy>(catalogs: &[Vec<T>], key: &T) -> Vec<Option<usize>> {
    catalogs
        .iter()
        .map(|catalog| match catalog.binary_search(key) {
            Ok(index) => Some(index),
            Err(index) => {
                if index == 0 {
                    None
                } else {
                    Some(index - 1)
                }
            }
        })
        .collect()
}

fn search_benchmark(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("create_searcher");

    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB, 32 * KB].iter() {
        let catalogs: Vec<_> = (0..40).map(|_| (0..*size as u64).collect()).collect();
        let searcher = FractionalCascading::new(&catalogs);

        let mut rng = rand::rngs::StdRng::seed_from_u64(6542728);
        let mut keys: Vec<_> = (0..*size as u64).collect();
        keys.shuffle(&mut rng);
        keys.truncate(50);

        group.bench_with_input(
            BenchmarkId::new("BinarySearch", size),
            &catalogs,
            |b, catalogs| {
                b.iter(|| {
                    for key in &keys {
                        find_with_binary_search(catalogs, key);
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("FractionalCascading", size),
            &searcher,
            |b, searcher| {
                b.iter(|| {
                    for key in &keys {
                        searcher.search(key);
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, search_benchmark);
criterion_main!(benches);
