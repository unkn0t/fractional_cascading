mod node;
mod fractional_cascading;

pub use fractional_cascading::FractionalCascading;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn one_catalog() {
        let catalogs = vec![vec![1, 2, 3, 4, 5]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(2)]);
        assert_eq!(searcher.search(&6), vec![Some(4)]);
        assert_eq!(searcher.search(&0), vec![None]);
    }

    #[test]
    fn two_catalogs() {
        let catalogs = vec![vec![1, 3, 6, 10], vec![2, 4, 5, 7, 8, 9]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(1), Some(0)]);
        assert_eq!(searcher.search(&6), vec![Some(2), Some(2)]);
        assert_eq!(searcher.search(&1), vec![Some(0), None]);
    }

    #[test]
    fn two_with_identical_catalogs() {
        let catalogs = vec![vec![1, 2, 4, 8], vec![0, 2, 4, 6]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(1), Some(1)]);
        assert_eq!(searcher.search(&4), vec![Some(2), Some(2)]);
        assert_eq!(searcher.search(&0), vec![None, Some(0)]);
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{thread_rng, SeedableRng, Rng};
    use rand::distributions::uniform::SampleRange;

    fn random_catalogs<R>(count: usize, size_range: R, seed: u64) -> Vec<Vec<i32>> 
    where R: SampleRange<usize> + Clone
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
            .map(|catalog| {
                match catalog.binary_search(key) {
                    Ok(index) => Some(catalog[index]),
                    Err(index) => if index == 0 { None } else { Some(catalog[index - 1]) },
                }
            })
            .collect()
    }

    #[test]
    fn hundred_catalogs() {
        for _ in 0..100 {
            let seed = thread_rng().gen();

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

    #[test]
    fn large_catalogs() {
        let seed = thread_rng().gen();

        let catalogs = random_catalogs(300, 1000..100000, seed);
        let searcher = FractionalCascading::new(&catalogs);

        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..100 {
            let key = rng.gen();
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