use std::borrow::Borrow;

#[derive(Clone, Debug)]
pub struct FCSearcher<T> {
    cats: Vec<Vec<Node<T>>>,
}

// ncur -> number of items less than value in current
// nnxt -> number of items less than value in next
#[derive(Clone, Copy, Debug)]
struct Node<T> {
    val: WithMax<T>,
    ncur: usize,
    nnxt: usize,
}

// FIXME: replace this struct with trait
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WithMax<T> {
    Val(T),
    Max,
}

impl<'val, T: Ord + Clone + 'val> FCSearcher<T> {
    pub fn new<'slc, I, S>(sources: I) -> Self
    where
        'val: 'slc,
        S: Borrow<[T]> + 'slc,
        I: IntoIterator<Item = &'slc S>,
        <I as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        let mut cats = Vec::new();
        let mut srcs = sources.into_iter().rev();

        if let Some(last_src) = srcs.next() {
            let mut last_cat = cat_from_src(last_src.borrow());

            for src in srcs {
                let new_last_cat = cat_merged_with_src(&last_cat, src.borrow());
                cats.push(last_cat);
                last_cat = new_last_cat;
            }

            cats.push(last_cat);
        }
        cats.reverse();

        Self { cats }
    }

    // TODO: return iterator
    pub fn search(&self, key: &T) -> Vec<usize> {
        let mut res = Vec::new();

        let pos = self.cats[0].partition_point(|node| node.val.as_ref() < WithMax::Val(key));
        let mut node = &self.cats[0][pos];
        res.push(node.ncur);

        for k in 1..self.cats.len() {
            if node.nnxt > 0 && self.cats[k][node.nnxt - 1].val.as_ref() >= WithMax::Val(key) {
                node = &self.cats[k][node.nnxt - 1];
            } else {
                node = &self.cats[k][node.nnxt];
            }

            res.push(node.ncur);
        }

        res
    }
}

fn cat_from_src<T: Clone + Eq>(src: &[T]) -> Vec<Node<T>> {
    let mut res = Vec::new();
    let mut sprev = 0;

    for sind in 0..src.len() {
        if src[sind] != src[sprev] {
            sprev = sind;
        }
        res.push(Node::new(WithMax::Val(src[sind].clone()), sprev, 0));
    }

    res.push(Node::new(WithMax::Max, src.len(), 0));
    res
}

fn cat_merged_with_src<T: Clone + Ord>(cat: &[Node<T>], src: &[T]) -> Vec<Node<T>> {
    let mut res = Vec::new();
    let mut sprev = 0;
    let mut cprev = 0;
    let mut sind = 0;

    for cind in 0..cat.len() - 1 {
        while sind < src.len() && WithMax::Val(&src[sind]) < cat[cind].val.as_ref() {
            if src[sind] != src[sprev] {
                sprev = sind;
            }

            if cat[cprev].val.as_ref() == WithMax::Val(&src[sind]) {
                res.push(Node::new(WithMax::Val(src[sind].clone()), sprev, cprev));
            } else {
                res.push(Node::new(WithMax::Val(src[sind].clone()), sprev, cind));
            }
            sind += 1;
        }

        if cat[cind].val != cat[cprev].val {
            cprev = cind;
        }

        if cind & 1 == 0 {
            res.push(Node::new(cat[cind].val.clone(), sind, cprev));
        }
    }

    while sind < src.len() {
        if src[sind] != src[sprev] {
            sprev = sind;
        }

        if cat[cprev].val.as_ref() == WithMax::Val(&src[sind]) {
            res.push(Node::new(WithMax::Val(src[sind].clone()), sprev, cprev));
        } else {
            res.push(Node::new(WithMax::Val(src[sind].clone()), sprev, cat.len()));
        }
        sind += 1;
    }

    res.push(Node::new(WithMax::Max, sind, cat.len()));
    res
}

impl<T> Node<T> {
    fn new(val: WithMax<T>, ncur: usize, nnxt: usize) -> Self {
        Self { val, ncur, nnxt }
    }
}

impl<T> WithMax<T> {
    #[inline]
    pub fn as_ref(&self) -> WithMax<&T> {
        match *self {
            Self::Val(ref x) => WithMax::Val(x),
            Self::Max => WithMax::Max,
        }
    }
}

// TODO: Better unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_catalog() {
        let catalogs = vec![[1, 2, 3, 4, 5]];
        let searcher = FCSearcher::new(&catalogs);
        assert_eq!(searcher.search(&3), &[2]);
        assert_eq!(searcher.search(&6), &[5]);
        assert_eq!(searcher.search(&0), &[0]);
    }

    #[test]
    fn many_catalogs() {
        let catalogs = vec![vec![1, 3, 6, 10], vec![2, 4, 5, 7, 8, 9]];
        let searcher = FCSearcher::new(&catalogs);
        assert_eq!(searcher.search(&3), &[1, 1]);
        assert_eq!(searcher.search(&6), &[2, 3]);
        assert_eq!(searcher.search(&1), &[0, 0]);
    }

    #[test]
    fn equal_elements() {
        let catalogs = vec![vec![1, 2, 4, 8], vec![0, 2, 4, 6]];
        let searcher = FCSearcher::new(&catalogs);
        assert_eq!(searcher.search(&3), &[2, 2]);
        assert_eq!(searcher.search(&4), &[2, 2]);
        assert_eq!(searcher.search(&0), &[0, 0]);
    }
}
