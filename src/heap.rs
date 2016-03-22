use std::cmp::Ordering;
use std::fmt;

pub struct Heap<T, F> {
    trees: Vec<Tree<T>>,
    by: F
}

impl<T, F> fmt::Debug for Heap<T, F>
    where T: fmt::Debug {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Heap(trees={:?}", &self.trees)
    }
}

impl<T, F> Heap<T, F>
    where F: Fn(&T, &T) -> Ordering {

    pub fn new(by: F) -> Self {
        Heap {
            trees: Vec::new(),
            by: by
        }
    }

    pub fn size(&self) -> usize {
        self.trees.iter().fold(0, |sz, tree| sz + tree.size())
    }

    pub fn add<IT>(&mut self, data: IT)
        where IT: Into<Tree<T>> {

        let tree_len = self.trees.len();
        if tree_len == 0 {
            self.trees.push(data.into())
        } else {
            let new_tree = data.into();
            if self.trees[tree_len - 1].accept(&new_tree, &self.by) {
                self.trees[tree_len - 1].add(new_tree);
            } else {
                self.trees.push(new_tree);
            }
        }
    }

    pub fn find_min(&mut self) -> Option<T> {
        if self.trees.len() == 0 {
            return None;
        }
        let mut smallest = 0usize;
        let mut idx = 1usize;
        loop {
            if idx == self.trees.len() {
                break;
            }
            let smaller = (&self.by)(&self.trees[smallest].node,
                                     &self.trees[idx].node) == Ordering::Greater;
            if smaller {
                smallest = idx;
                idx += 1;
            } else {
                if self.trees[smallest].accept(&self.trees[idx], &self.by) {
                    let tree = self.trees.swap_remove(idx);
                    self.trees[smallest].add(tree);
                } else {
                    idx += 1;
                }
            }
        }
        let mut smallest_tree = self.trees.swap_remove(smallest);
        match smallest_tree.subtrees {
            None => (),
            Some(ref mut subtrees) => {
                println!("Adding {} subtrees", subtrees.len());
                self.trees.append(subtrees);
            }
        }
        println!("Number of trees: {}, found smallest at: {}", self.trees.len(), smallest);
        Some(smallest_tree.node)
    }
}

impl<T> From<T> for Tree<T> {
    fn from(from: T) -> Tree<T> {
        Tree::new(from)
    }
}

const MAX_SUBTREES_SIZE:usize = 8;

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

    pub fn size(&self) -> usize {
        match self.subtrees {
            None => 1,
            Some(ref subtrees) => {
                subtrees.iter().fold(1, |sz, tree| sz + tree.size())
            }
        }
    }

    pub fn add(&mut self, data: Tree<T>) {
        match self.subtrees {
            None => {
                self.subtrees = Some({
                    let mut s = Vec::with_capacity(MAX_SUBTREES_SIZE);
                    s.push(data);
                    s
                });
            },
            Some(ref mut subtrees) => {
                subtrees.push(data);
            }
        }
    }

    pub fn accept<F>(&self, data: &Tree<T>, cmp: &F) -> bool
        where F: Fn(&T, &T) -> Ordering {

        match cmp(&self.node, &data.node) {
            Ordering::Greater => false,
            Ordering::Equal | Ordering::Less => {
                match self.subtrees {
                    None => true,
                    Some(ref subtrees) => !(subtrees.len() >= MAX_SUBTREES_SIZE)
                }
            }
        }
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

        assert_eq!(Some(1), heap.find_min());
        assert_eq!(Some(2), heap.find_min());
        assert_eq!(None, heap.find_min());
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

        assert_eq!(Some(1), heap.find_min());
        assert_eq!(Some(2), heap.find_min());
        assert_eq!(Some(3), heap.find_min());
        assert_eq!(Some(10), heap.find_min());
        assert_eq!(Some(11), heap.find_min());
        assert_eq!(Some(12), heap.find_min());
        assert_eq!(Some(20), heap.find_min());
    }
}
