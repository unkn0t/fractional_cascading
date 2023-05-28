#[derive(Debug)]
pub enum NodeData<'a, T> {
    Real(usize),                // index in original catalog
    Synthetic(&'a T),           // ref on real element
    Fake(&'a [T]),              // catalog
}

// match node.data { 
//      Real => prev = Synthetic(1..) or Fake(0)
//      Synthetic => prev = Real(1..) or Fake(0) 
//      Fake => prev = Fake(0) 
// }

#[derive(Debug)]
pub struct Node<'a, T> {
    pub data: NodeData<'a, T>,      // data
    pub prev: usize,                // index of previous in same augmented catalog
    pub bridge: usize,              // index of bridge in next augmented catalog
}

impl<'a, T> Node<'a, T> {
    pub fn real(data: usize, prev: usize) -> Self {
        Self { 
            data: NodeData::Real(data),
            prev,
            bridge: 0,
        }
    }

    pub fn synthetic(data: &'a T, prev: usize, bridge: usize) -> Self {
        Self { 
            data: NodeData::Synthetic(data),
            prev,
            bridge,
        }
    }

    pub fn fake(catalog: &'a [T]) -> Self {
        Self {
            data: NodeData::Fake(catalog),
            prev: 0,
            bridge: 0,
        }
    }

    pub fn is_real(&self) -> bool {
        match self.data {
            NodeData::Real(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_fake(&self) -> bool {
        match self.data {
            NodeData::Fake(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_synthetic(&self) -> bool {
        match self.data {
            NodeData::Synthetic(_) => true,
            _ => false
        }
    }

    pub fn value<'b, 'c>(&'c self, fake_node: &'b Node<'a, T>) -> Option<&'a T> {
        let catalog = match fake_node.data {
            NodeData::Fake(catalog) => catalog,
            _ => panic!("Invalid node")
        };
        match self.data {
            NodeData::Real(index) => catalog.get(index),
            NodeData::Synthetic(data) => Some(data),
            NodeData::Fake(_) => None,
        }
    }

    pub fn closest_real_index(&self, augmented: &[Node<'a, T>]) -> Option<usize> {
        match self.data {
            NodeData::Real(index) => Some(index),
            NodeData::Synthetic(_) => augmented[self.prev].closest_real_index(augmented),
            NodeData::Fake(_) => None,
        }
    }
}

pub fn merge_catalog_with_augmented<'a, 'b, T: Ord>(catalog: &'a [T], augmented: &'b [Node<'a, T>]) -> Vec<Node<'a, T>> {
    let catalog_len = catalog.len();
    let augmented_len = augmented.len();
    
    // reserve 1 additional slot for fake element
    let mut result = Vec::with_capacity(catalog_len + (augmented_len - 1) / 2 + 1);    
    result.push(Node::fake(catalog));

    // two pointers algorithm
    let mut last_real = 0;
    let mut last_synthetic = 0;
    let mut catalog_index = 0;

    for (augmented_index, node) in augmented.iter().enumerate().skip(1).step_by(2) {    
        while catalog_index < catalog_len && catalog.get(catalog_index) < node.value(&augmented[0]) {
            result.push(Node::real(catalog_index, last_synthetic));
            catalog_index += 1;
            last_real = result.len() - 1;
        }  

        result.push(Node::synthetic(node.value(&augmented[0]).unwrap(), last_real, augmented_index)); 
        last_synthetic = result.len() - 1;
    }

    while catalog_index < catalog_len {
        result.push(Node::real(catalog_index, last_synthetic));
        catalog_index += 1;
    } 

    result
}