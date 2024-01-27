use fractional_cascading::FractionalCascading;

use rand::distributions::uniform::SampleRange;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[test]
fn main() {
    for _ in 0..100 {
        let seed = rand::thread_rng().gen();

        let catalogs = random_catalogs(100, 10..1000, seed);
        let searcher = FractionalCascading::new(&catalogs);

        for key in -200..200 {
            let debug_log = format!("\nTesting seed: {seed}\nSearching key: {key}");
            let s: Vec<_> = searcher
                .search(&key)
                .iter()
                .enumerate()
                .map(|(i, x)| x.map(|y| catalogs[i][y]))
                .collect();
            assert_eq!(s, find_with_binary_search(&catalogs, &key), "{debug_log}");
        }
    }
}

fn random_catalogs<R>(count: usize, size_range: R, seed: u64) -> Vec<Vec<i32>>
where
    R: SampleRange<usize> + Clone,
{
    let mut rng = StdRng::seed_from_u64(seed);

    let mut catalogs = Vec::with_capacity(count);
    for _ in 0..count {
        let catalog_size = rng.gen_range(size_range.clone());
        let mut catalog = Vec::with_capacity(catalog_size);

        for _ in 0..catalog_size {
            catalog.push(rng.gen_range(-100..100));
        }
        catalog.sort();
        catalogs.push(catalog);
    }

    catalogs
}

fn find_with_binary_search<T: Ord + Copy>(catalogs: &[Vec<T>], key: &T) -> Vec<Option<T>> {
    catalogs
        .iter()
        .map(|catalog| match catalog.binary_search(key) {
            Ok(index) => Some(catalog[index]),
            Err(index) => {
                if index == 0 {
                    None
                } else {
                    Some(catalog[index - 1])
                }
            }
        })
        .collect()
}
