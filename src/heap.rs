use std::cmp::Ordering;
use std::fmt;
use std::mem;

pub struct Heap<T, F> {
    tree: Option<Tree<T>>,
    by: F
}

impl<T, F> fmt::Debug for Heap<T, F>
    where T: fmt::Debug {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Heap(at={:p},tree={:?})", self, &self.tree)
    }
}

impl<T, F> Heap<T, F>
    where F: Fn(&T, &T) -> Ordering {

    pub fn new(by: F) -> Self {
        Heap {
            tree: None,
            by: by
        }
    }

    pub fn size(&self) -> usize {
        match self.tree {
            None => 0,
            Some(ref tree) => tree.size()
        }
    }

    pub fn add<IT>(&mut self, data: IT)
        where IT: Into<Tree<T>> {

        match self.tree {
            None => {
                self.tree = Some(data.into());
            },
            Some(ref mut tree) => {
                tree.merge(data.into(), &self.by);
            }
        }
    }

    pub fn take(&mut self) -> Option<T> {
        if self.tree.is_none() {
            return None;
        }
        Some(take_from_tree(self))
    }
}

fn take_from_tree<T, F>(orig_opt: &mut Heap<T, F>) -> T
    where F: Fn(&T, &T) -> Ordering {

    let mut orig_tree_opt = None;
    mem::swap(&mut orig_tree_opt, &mut orig_opt.tree);
    let orig_tree = orig_tree_opt.expect("A tree");
    let val = orig_tree.node;
    let new_tree = match orig_tree.subtrees {
        None => None,
        Some(mut subtrees) => {
            match subtrees.pop() {
                None => None,
                Some(mut tree) => {
                    for subtree in subtrees.drain(..) {
                        tree.merge(subtree, &orig_opt.by)
                    }
                    Some(tree)
                }
            }
        }
    };
    orig_opt.tree = new_tree;
    val
}

impl<T> From<T> for Tree<T> {
    fn from(from: T) -> Tree<T> {
        Tree::new(from)
    }
}

const MAX_SUBTREES_SIZE:usize = 256;

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

    pub fn merge<F>(&mut self, mut data: Tree<T>, by: &F)
        where F: Fn(&T, &T) -> Ordering {

        if by(&self.node, &data.node) == Ordering::Greater {
            mem::swap(&mut self.node, &mut data.node);
            match data.subtrees {
                None => (),
                Some(ref mut subtrees) => {
                    for subtree in subtrees.drain(..) {
                        self.add(subtree, by);
                    }
                }
            }
            self.add(data, by);
        } else {
            self.add(data, by);
        }
    }

    fn add<F>(&mut self, data: Tree<T>, by: &F)
        where F: Fn(&T, &T) -> Ordering {

        match self.subtrees {
            None => {
                self.subtrees = Some({
                    let mut s = Vec::with_capacity(MAX_SUBTREES_SIZE);
                    s.push(data);
                    s
                });
            },
            Some(ref mut subtrees) => {
                if subtrees.len() < MAX_SUBTREES_SIZE {
                    subtrees.push(data);
                } else {
                    let last_idx = subtrees.len() - 1;
                    subtrees[last_idx].merge(data, by);
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
