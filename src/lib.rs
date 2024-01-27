mod node;

use node::{merge_catalog_with_augmented, Node};

#[derive(Debug)]
pub struct FractionalCascading<'a, T> {
    augmented_catalogs: Vec<Vec<Node<'a, T>>>,
}

impl<'a, T: Ord> FractionalCascading<'a, T> {
    pub fn new(catalogs: &'a [Vec<T>]) -> Self {
        debug_assert_ne!(catalogs.len(), 0, "Catalogs must have elements");

        let catalogs_len = catalogs.len();
        let mut augmented_catalogs = Vec::with_capacity(catalogs_len);

        // Use unwrap instead unwrap_unchecked
        // so function will be safe to call in release
        augmented_catalogs.push(merge_catalog_with_augmented(
            catalogs.last().unwrap(),
            &[Node::fake(&catalogs[catalogs_len - 1])],
        ));

        for catalog in catalogs.iter().rev().skip(1) {
            augmented_catalogs.push(merge_catalog_with_augmented(catalog.as_slice(), unsafe {
                augmented_catalogs.last().unwrap_unchecked()
            }));
        }

        augmented_catalogs.reverse();

        Self { augmented_catalogs }
    }

    pub fn search(&self, key: &T) -> Vec<Option<usize>> {
        let catalogs_len = self.augmented_catalogs.len();
        let mut result = Vec::with_capacity(catalogs_len);

        let mut index = self.augmented_catalogs[0].partition_point(
            |node| unsafe { node.value(&self.augmented_catalogs[0][0]) } <= Some(key),
        ) - 1;

        let mut node = &self.augmented_catalogs[0][index];

        result.push(node.closest_real_index(&self.augmented_catalogs[0]));

        if node.is_real() {
            index = node.prev;
            node = &self.augmented_catalogs[0][index];
        }

        for i in 1..catalogs_len {
            // go to next catalog
            index = node.bridge;
            node = &self.augmented_catalogs[i][index];

            // try move to next node
            if let Some(next_node) = self.augmented_catalogs[i].get(index + 1) {
                if unsafe { next_node.value(&self.augmented_catalogs[i][0]) } <= Some(key) {
                    node = next_node;
                }
            }

            // pushing current node
            result.push(node.closest_real_index(&self.augmented_catalogs[i]));

            if node.is_real() {
                index = node.prev;
                node = &self.augmented_catalogs[i][index];
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_catalog() {
        let catalogs = vec![vec![1, 2, 3, 4, 5]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(2)]);
        assert_eq!(searcher.search(&6), vec![Some(4)]);
        assert_eq!(searcher.search(&0), vec![None]);
    }

    #[test]
    fn many_catalogs() {
        let catalogs = vec![vec![1, 3, 6, 10], vec![2, 4, 5, 7, 8, 9]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(1), Some(0)]);
        assert_eq!(searcher.search(&6), vec![Some(2), Some(2)]);
        assert_eq!(searcher.search(&1), vec![Some(0), None]);
    }

    #[test]
    fn equal_elements() {
        let catalogs = vec![vec![1, 2, 4, 8], vec![0, 2, 4, 6]];
        let searcher = FractionalCascading::new(&catalogs);
        assert_eq!(searcher.search(&3), vec![Some(1), Some(1)]);
        assert_eq!(searcher.search(&4), vec![Some(2), Some(2)]);
        assert_eq!(searcher.search(&0), vec![None, Some(0)]);
    }
}

