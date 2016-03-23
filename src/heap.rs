use std::cmp::Ordering;
use std::fmt;
use std::mem;
use std::collections::HashMap;

const DEFAULT_HEAP:usize = 16;

pub struct Heap<T, F> {
    /// The trees
    trees: Vec<Tree<T>>,

    /// The position of the minimum value
    min: Option<usize>,

    /// Function for determining lowest value
    by: F
}

impl<T, F> fmt::Debug for Heap<T, F>
    where T: fmt::Debug {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Heap(at={:p},trees={:?})", self, &self.trees)
    }
}

impl<T, F> Heap<T, F>
    where F: Fn(&T, &T) -> Ordering {

    pub fn new(by: F) -> Self {
        Heap {
            trees: Vec::with_capacity(DEFAULT_HEAP),
            min: None,
            by: by
        }
    }

    pub fn size(&self) -> usize {
        self.trees.len()
    }

    pub fn add(&mut self, data: T) {
        match self.min {
            None => { self.min = Some(0); },
            Some(min) => {
                if (self.by)(&self.trees[min].node, &data) == Ordering::Greater {
                    self.min = Some(self.trees.len());
                }
            }
        }
        self.trees.push(data.into());
    }

    pub fn take(&mut self) -> Option<T> {
        if self.min.is_none() {
            let success = self.find_min();
            if !success {
                return None
            }
        }
        let mut lowest_tree = self.trees.swap_remove(self.min.expect("min val"));
        match lowest_tree.subtrees {
            None => (),
            Some(ref mut subtrees) => { self.trees.append(subtrees); }
        }
        self.min = None;
        Some(lowest_tree.node)
    }

    fn find_min(&mut self) -> bool {
        if self.trees.len() == 0 {
            return false
        }
        let mut prev_pos = HashMap::new();
        let mut idx = 0;
        let mut min = 0;
        loop {
            let order = self.trees[idx].order();
            if prev_pos.contains_key(&order) {
                let pos:usize = prev_pos.remove(&order).expect("pos");
                let idx_tree = self.trees.swap_remove(idx);
                self.trees[pos].merge(idx_tree, &self.by);
                prev_pos.insert(order + 1, pos);
                if (self.by)(&self.trees[min].node, &self.trees[pos].node) == Ordering::Greater {
                    min = pos;
                }
            } else {
                prev_pos.insert(order, idx);
                if (self.by)(&self.trees[min].node, &self.trees[idx].node) == Ordering::Greater {
                    min = idx;
                }
                idx += 1;
            }
            if idx >= self.trees.len() {
                break;
            }
        }
        self.min = Some(min);
        true
    }
}

impl<T> From<T> for Tree<T> {
    fn from(from: T) -> Tree<T> {
        Tree::new(from)
    }
}

const DEFAULT_TREE_CAPACITY:usize = 4;

#[derive(Debug)]
pub struct Tree<T> {
    node: T,
    subtrees: Option<Vec<Tree<T>>>
}

impl<T> Tree<T> {
    pub fn new(data: T) -> Self {
        Tree {
            node: data,
            subtrees: None
        }
    }

    pub fn order(&self) -> usize {
        match self.subtrees {
            None => 0,
            Some(ref subtrees) => subtrees.len()
        }
    }

    pub fn merge<F>(&mut self, mut other_tree: Tree<T>, by: F)
        where F: Fn(&T, &T) -> Ordering {

        if by(&self.node, &other_tree.node) == Ordering::Greater {
            mem::swap(self, &mut other_tree);
        }

        if self.subtrees.is_none() {
            self.subtrees = Some(Vec::with_capacity(DEFAULT_TREE_CAPACITY));
        }

        self.subtrees.as_mut().expect("subtrees").push(other_tree);
    }
}

#[cfg(test)]
mod tests {
    use super::Heap;

    #[test]
    fn add_test() {
        let mut heap = Heap::new(Ord::cmp);
        heap.add(1u64);
        heap.add(2u64);

        println!("{:?}", heap);
        assert_eq!(Some(1), heap.take());
        println!("{:?}", heap);
        assert_eq!(Some(2), heap.take());
        println!("{:?}", heap);
        assert_eq!(None, heap.take());
        println!("Finished *** {:?}", heap);
    }

    #[test]
    fn other_test() {
        let mut heap = Heap::new(Ord::cmp);
        heap.add(10u64);
        heap.add(11u64);
        heap.add(12u64);
        heap.add(1u64);
        heap.add(2u64);
        heap.add(3u64);
        heap.add(20u64);

        println!("{:?}", heap);
        assert_eq!(Some(1), heap.take());
        println!("{:?}", heap);
        assert_eq!(Some(2), heap.take());
        println!("{:?}", heap);
        assert_eq!(Some(3), heap.take());
        println!("{:?}", heap);
        assert_eq!(Some(10), heap.take());
        println!("{:?}", heap);
        assert_eq!(Some(11), heap.take());
        println!("{:?}", heap);
        assert_eq!(Some(12), heap.take());
        println!("{:?}", heap);
        assert_eq!(Some(20), heap.take());
        println!("Finished *** {:?}", heap);
    }
}
