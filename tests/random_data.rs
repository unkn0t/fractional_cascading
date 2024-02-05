use std::ops::Range;

use fractional_cascading::FCSearcher;

use rand::distributions::{Uniform, Distribution};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

const SEED: Option<u64> = None;
const N_ITERATIONS: usize = 20;
const N_SOURCES: usize = 100;
const N_KEYS: usize = 20;
const MAX_SRC_LEN: usize = 100;
const VAL_RANGE: Range<i32> = -200..201;

#[test]
fn main() {
    if let Some(seed) = SEED {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut sources = Vec::with_capacity(N_SOURCES);
        let val_range = Uniform::from(VAL_RANGE);

        for _ in 0..N_SOURCES {
            let len = rng.gen_range(0..=MAX_SRC_LEN);
            let mut src: Vec<_> = val_range
                .sample_iter(&mut rng)
                .take(len)
                .collect();
            src.sort();
            sources.push(src);
        }

        let searcher = FCSearcher::new(&sources);

        for _ in 0..N_KEYS {
            let key = rng.gen_range(VAL_RANGE);
            for (ind, pos) in searcher.search(&key).into_iter().enumerate() {
                assert!(fit_in_pos(&sources[ind], pos, &key));
            }
        }
    } else {
        for it in 0..N_ITERATIONS {
            let seed = rand::thread_rng().gen();
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut sources = Vec::with_capacity(N_SOURCES);
            let val_range = Uniform::from(VAL_RANGE);

            for _ in 0..N_SOURCES {
                let len = rng.gen_range(0..=MAX_SRC_LEN);
                let mut src: Vec<_> = val_range
                    .sample_iter(&mut rng)
                    .take(len)
                    .collect();
                src.sort();
                sources.push(src);
            }

            let searcher = FCSearcher::new(&sources);

            for _ in 0..N_KEYS {
                let key = rng.gen_range(VAL_RANGE);
                for (ind, pos) in searcher.search(&key).into_iter().enumerate() {
                    assert!(
                        fit_in_pos(&sources[ind], pos, &key),
                        "Iteration #{it}, seed: {seed}, key: {key}"
                    );
                }
            }
        }
    }
}

fn fit_in_pos<T: Ord>(src: &[T], pos: usize, key: &T) -> bool {
    let mut res = true;

    if pos < src.len() {
        res = src[pos] >= *key;
    }

    if res && pos > 0 {
        res = src[pos - 1] < *key;
    }

    res
}
