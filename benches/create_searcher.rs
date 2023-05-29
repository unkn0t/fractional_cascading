use fractional_cascading::FractionalCascading;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

fn create_searcher(catalogs: &[Vec<u64>]) -> FractionalCascading<u64> {
    FractionalCascading::new(catalogs)
}

fn create_searcher_benchmark(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("create_searcher");
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(50);

    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
        let catalogs: Vec<_> = (0..20).map(|_| (0..*size as u64).collect()).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &catalogs,
            |b, catalogs| {
                b.iter(|| create_searcher(catalogs));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, create_searcher_benchmark);
criterion_main!(benches);
