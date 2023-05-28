mod node;

use std::cmp::Ordering; 
use std::fmt::Display;

#[derive(Debug)]
enum Key<T> {
    Real {
        value: T,
        prev: usize,
        next: usize,
    }, 
    Synthetic {
        value: T,
        prev: usize,
        next: usize,
        bridge: usize,
    },
    PlusInfinity {
        _prev: usize,
        bridge: usize,
    },
    MinusInfinity {
        next: usize,
        bridge: usize,
    },
}

impl<T: PartialEq> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Real { value, .. }, Self::Real { value: val, .. }) => value == val,
            (Self::Real { value, .. }, Self::Synthetic { value: val, .. }) => value == val,
            (Self::Synthetic { value, .. }, Self::Real { value: val, .. }) => value == val,
            (Self::Synthetic { value, .. }, Self::Synthetic { value: val, .. }) => value == val,
            (Self::MinusInfinity { .. }, Self::MinusInfinity { .. }) => true,
            (Self::PlusInfinity { .. }, Self::PlusInfinity { .. }) => true,
            _ => false,
        }
    }
}

impl<T: Eq> Eq for Key<T> {}

impl<T: PartialOrd> PartialOrd for Key<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Real { value, .. }, Self::Real { value: val, .. }) => value.partial_cmp(val),
            (Self::Real { value, .. }, Self::Synthetic { value: val, .. }) => value.partial_cmp(val),
            (Self::Synthetic { value, .. }, Self::Real { value: val, .. }) => value.partial_cmp(val),
            (Self::Synthetic { value, .. }, Self::Synthetic { value: val, .. }) => value.partial_cmp(val),
            (Self::MinusInfinity { .. }, Self::MinusInfinity { .. }) => Some(Ordering::Equal),
            (Self::PlusInfinity { .. }, Self::PlusInfinity { .. }) => Some(Ordering::Equal),
            (Self::MinusInfinity { .. }, Self::Real { .. }) => Some(Ordering::Less),
            (Self::MinusInfinity { .. }, Self::Synthetic { .. }) => Some(Ordering::Less),
            (Self::MinusInfinity { .. }, Self::PlusInfinity { .. }) => Some(Ordering::Less),
            (Self::Real { .. }, Self::PlusInfinity { .. }) => Some(Ordering::Less),
            (Self::Synthetic { .. }, Self::PlusInfinity { .. }) => Some(Ordering::Less),
            _ => Some(Ordering::Greater),
        }
    }
}

impl<T: Ord> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}

impl<T> Key<T> {
    fn real(value: T, prev: usize) -> Self { 
        Self::Real { value, prev, next: 0 }
    }

    fn synthetic(value: T, prev: usize, bridge: usize) -> Self {
        Self::Synthetic { value, prev, next: 0, bridge}
    }

    fn plus_inf(_prev: usize, bridge: usize) -> Self {
        Self::PlusInfinity { _prev, bridge }
    }

    fn minus_inf(bridge: usize) -> Self {
        Self::MinusInfinity { next: 0, bridge }
    }

    fn value(&self) -> Option<&T> {
        match self {
            Self::Real { value, .. } | Self::Synthetic { value, .. } => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct FractionalCascading<T> {
    catalogs: Vec<Vec<Key<T>>>,
}

fn merge_catalogs<T: Ord + Clone>(target: &Vec<T>, augmented: &Vec<Key<T>>) -> Vec<Key<T>> {
    let target_len = target.len();
    let augmented_len = augmented.len();
    
    // reserve 2 additional slots for PlusInf and MinusInf
    let mut result = Vec::with_capacity(target_len + augmented_len + 2);    
    
    // Synthetic or MinusInf <- Real -> Any or PlusInf
    // Real or MinusInf <- Synthetic -> Any or PlusInf
    // Any <- PlusInf 
    // MinusInf -> Any

    // augmented must have MinusInf and PlusInf
    debug_assert!(augmented_len >= 2);
    let (_, augmented) = unsafe { augmented.split_first().unwrap_unchecked() };
    let (_, augmented) = unsafe { augmented.split_last().unwrap_unchecked() };

    // push MinusInf
    result.push(Key::minus_inf(0));

    let mut last_real = 0;
    let mut last_synthetic = 0;

    // current target element
    let mut tar_ind = 0;

    for (aug_ind, key) in augmented.iter().enumerate().step_by(2) {    
        match key {
            Key::Real { value, .. } | Key::Synthetic { value, .. } => {
                while tar_ind < target_len && target[tar_ind] < *value {
                    result.push(Key::real(target[tar_ind].clone(), last_synthetic));
                    tar_ind += 1;
                    last_real = result.len() - 1;
                }  

                result.push(Key::synthetic(value.clone(), last_real, aug_ind + 1)); 
                last_synthetic = result.len() - 1;
            },
            _ => panic!("Unexpected key"),
        }
    }

    while tar_ind < target_len {
        result.push(Key::real(target[tar_ind].clone(), last_synthetic));
        tar_ind += 1;
    } 

    // push PlusInf
    result.push(Key::plus_inf(result.len() - 1, augmented_len - 1));

    // walk backwards and set key.next field
    // let mut last_real = result.len() - 1;
    let mut last_any = result.len() - 1;

    for key in result.iter_mut().rev() {
        match key {
            Key::Real { next, .. } => {
                *next = last_any;
                last_any -= 1;
            },
            Key::MinusInfinity { next, .. } => {
                *next = last_any;
            },
            Key::Synthetic { next, .. } => {
                *next = last_any;
                last_any -= 1;
            }
            _ => {},
        }
    }

    result
}

impl<T: Ord + Clone> FractionalCascading<T> {
    pub fn new(catalogs: &[Vec<T>]) -> Self {
        let mut augmented_catalogs = Vec::with_capacity(catalogs.len());

        let dummy = vec![Key::minus_inf(0), Key::plus_inf(0, 0)];
        for catalog in catalogs.iter().rev() {
            let augmented_catalog = augmented_catalogs.last().unwrap_or(&dummy);
            augmented_catalogs.push(merge_catalogs(catalog, augmented_catalog));
        }

        augmented_catalogs.reverse();

        Self { catalogs: augmented_catalogs }
    }

    pub fn search(&self, real_key: &T) -> Vec<Option<T>> {
        // let debug_value = std::mem::discriminant(&Key::synthetic(real_key.clone(), 0, 0));

        if self.catalogs.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.catalogs.len());

        let real_key = Key::real(real_key.clone(), 0);
        
        let ind = self.catalogs[0].partition_point(|k| k <= &real_key) - 1;

        let mut key = &self.catalogs[0][ind];

        // key -> MinusInf -> (None)
        // key -> Synthetic -> (Prev)
        // key -> (Real) -> Prev

        match key {
            Key::MinusInfinity { .. } => result.push(None),
            Key::Synthetic { prev, .. } => result.push(self.catalogs[0][*prev].value().cloned()),
            Key::Real { value, prev, .. } => { 
                result.push(Some(value.clone())); 
                key = &self.catalogs[0][*prev];
            },
            _ => panic!("Unexpected key"),
        }

        for i in 1..self.catalogs.len() {
            // Go Down
            
            match key {
                Key::Real { .. } => panic!("Unexpected key"),
                Key::MinusInfinity { bridge, .. } | 
                Key::Synthetic { bridge, .. } | 
                Key::PlusInfinity { bridge, .. } => key = &self.catalogs[i][*bridge],
            }
            
            // try move to next
            match key {
                Key::Real { next, .. } | Key::Synthetic { next, .. } => {
                    if self.catalogs[i][*next] <= real_key {
                        key = &self.catalogs[i][*next];
                    }
                }   
                _ => {},
            }

            // key -> MinusInf -> (None)
            // key -> Synthetic -> (Prev)
            // key -> (Real) -> Prev

            match key {
                Key::MinusInfinity { .. } => result.push(None),
                Key::Synthetic { prev, .. } => result.push(self.catalogs[i][*prev].value().cloned()),
                Key::Real { value, prev, .. } => { 
                    result.push(Some(value.clone())); 
                    key = &self.catalogs[i][*prev];
                },
                _ => panic!("Unexpected key"),
            }
        } 
        
        result
    }
}

impl<T: Display> Display for FractionalCascading<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for catalog in &self.catalogs {
            for key in catalog {
                match key {
                    Key::Real { value, prev, next } => write!(f, "\x1b[34m{:2}/{:2}/{:2} ", value, prev, next)?,
                    Key::Synthetic { value, prev, next, bridge } => write!(f, "\x1b[31m{:2}/{:2}/{:2}/{:2} ", value, prev, next, bridge)?,
                    Key::MinusInfinity { .. } | Key::PlusInfinity { .. } => write!(f, "\x1b[32m{:2} ", '∞')?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn one_catalog() {
        let catalogs = vec![vec![1, 2, 3, 4, 5]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(3)]);
        assert_eq!(searcher.search(&6), vec![Some(5)]);
        assert_eq!(searcher.search(&0), vec![None]);
    }

    #[test]
    fn two_catalogs() {
        let catalogs = vec![vec![1, 3, 6, 10], vec![2, 4, 5, 7, 8, 9]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(3), Some(2)]);
        assert_eq!(searcher.search(&6), vec![Some(6), Some(5)]);
        assert_eq!(searcher.search(&1), vec![Some(1), None]);
    }

    #[test]
    fn two_with_identical_catalogs() {
        let catalogs = vec![vec![1, 2, 4, 8], vec![0, 2, 4, 6]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(2), Some(2)]);
        assert_eq!(searcher.search(&4), vec![Some(4), Some(4)]);
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

    fn find_with_binary_search<T: Ord + Clone>(catalogs: &[Vec<T>], key: &T) -> Vec<Option<T>> {
        catalogs
            .iter()
            .map(|catalog| {
                match catalog.binary_search(key) {
                    Ok(index) => Some(catalog[index].clone()),
                    Err(index) => if index == 0 { None } else { Some(catalog[index - 1].clone()) },
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
                let debug_log = format!("Testing seed: {seed}\nSearching key: {key}");
                assert_eq!(searcher.search(&key), find_with_binary_search(&catalogs, &key), "{debug_log}");
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
            let debug_log = format!("Testing seed: {seed}\nSearching key: {key}");
            assert_eq!(searcher.search(&key), find_with_binary_search(&catalogs, &key), "{debug_log}");
        }
    }
}