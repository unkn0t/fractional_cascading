#[derive(Debug)]
pub struct FractionalCascading<'a, T> {
    augmented_catalogs: Vec<Vec<Node<'a, T>>>,
}

#[derive(Debug)]
enum NodeData<'a, T> {
    Real(usize),      // index in original catalog
    Synthetic(&'a T), // ref on real element
    Fake(*const T),   // catalog
}

#[derive(Debug)]
struct Node<'a, T> {
    data: NodeData<'a, T>, // data
    prev: usize,           // index of previous in same augmented catalog
    bridge: usize,         // index of bridge in next augmented catalog
}

impl<'a, T: Ord> FractionalCascading<'a, T> {
    pub fn new(catalogs: &'a [Vec<T>]) -> Self {
        debug_assert_ne!(catalogs.len(), 0, "Catalogs must have elements");

        let catalogs_len = catalogs.len();
        let mut augmented_catalogs = Vec::with_capacity(catalogs_len);

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

impl<'a, T> Node<'a, T> {
    fn real(data: usize, prev: usize) -> Self {
        Self {
            data: NodeData::Real(data),
            prev,
            bridge: 0,
        }
    }

    fn synthetic(data: &'a T, prev: usize, bridge: usize) -> Self {
        Self {
            data: NodeData::Synthetic(data),
            prev,
            bridge,
        }
    }

    fn fake(catalog: &'a [T]) -> Self {
        Self {
            data: NodeData::Fake(catalog.as_ptr()),
            prev: 0,
            bridge: 0,
        }
    }

    #[inline]
    fn is_real(&self) -> bool {
        matches!(self.data, NodeData::Real(_))
    }

    unsafe fn value<'b, 'c>(&'c self, fake_node: &'b Node<'a, T>) -> Option<&'a T> {
        let catalog = match fake_node.data {
            NodeData::Fake(catalog) => catalog,
            _ => panic!("Invalid node"),
        };

        match self.data {
            NodeData::Real(index) => Some(&*catalog.add(index)),
            NodeData::Synthetic(data) => Some(data),
            NodeData::Fake(_) => None,
        }
    }

    #[inline]
    fn closest_real_index(&self, augmented: &[Node<'a, T>]) -> Option<usize> {
        match self.data {
            NodeData::Real(index) => Some(index),
            NodeData::Synthetic(_) => augmented[self.prev].closest_real_index(augmented),
            NodeData::Fake(_) => None,
        }
    }
}

fn merge_catalog_with_augmented<'a, T: Ord>(
    catalog: &'a [T],
    augmented: &[Node<'a, T>],
) -> Vec<Node<'a, T>> {
    let catalog_len = catalog.len();
    let augmented_len = augmented.len();

    // reserve 1 additional slot for fake element
    let mut result = Vec::with_capacity(catalog_len + (augmented_len + 1) / 2);
    result.push(Node::fake(catalog));

    // two pointers algorithm
    let mut last_real = 0;
    let mut last_synthetic = 0;
    let mut catalog_index = 0;

    for (augmented_index, node) in augmented.iter().enumerate().skip(1).step_by(2) {
        while catalog_index < catalog_len
            && catalog.get(catalog_index) < unsafe { node.value(&augmented[0]) }
        {
            result.push(Node::real(catalog_index, last_synthetic));
            catalog_index += 1;
            last_real = result.len() - 1;
        }

        result.push(Node::synthetic(
            unsafe { node.value(&augmented[0]).unwrap_unchecked() },
            last_real,
            augmented_index,
        ));
        last_synthetic = result.len() - 1;
    }

    while catalog_index < catalog_len {
        result.push(Node::real(catalog_index, last_synthetic));
        catalog_index += 1;
    }

    result
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
