#![crate_type = "lib"]
#![crate_name = "lazysort"]

extern crate test;

use std::rand::{task_rng, Rng};

use test::Bencher;

pub struct LazySortIterator<'a, T> {
    work: Vec<Vec<T>>,
    by: |&T, &T|:'a -> Ordering
}

impl <'a, T> LazySortIterator<'a, T> {
    fn new(work: Vec<T>, by: |&T, &T|:'a -> Ordering) -> LazySortIterator<'a, T> {
        LazySortIterator {
            work: vec![work],
            by: by
        }
    }
}

pub trait Sorted<'a, O: Ord> {
    fn sorted(&'a mut self) -> LazySortIterator<O>;
}

pub trait SortedBy<'a, T> {
    fn sorted_by(&'a mut self, |&T, &T|:'a -> Ordering) -> LazySortIterator<T>;
}

impl <'a, O: Ord, I: Iterator<O>> Sorted<'a, O> for I {
    fn sorted(&'a mut self) -> LazySortIterator<O> {
        LazySortIterator::new(self.collect(),
                              |a, b| a.cmp(b))
    }
}

impl <'a, T, I: Iterator<T>> SortedBy<'a, T> for I {
    fn sorted_by(&'a mut self, by: |&T, &T|:'a -> Ordering) -> LazySortIterator<T> {
        LazySortIterator::new(self.collect(), by)
    }
}

impl <'a, T> Iterator<T> for LazySortIterator<'a, T> {
    fn next(&mut self) -> Option<T> {
        match self.work.pop() {
            Some(mut next_work) => {
                let pivot = next_work.remove(0).unwrap();
                let (before, after) = next_work.partition(|v| (self.by)(v, &pivot) == Less);
                if before.len() == 0 {
                    if after.len() > 0 {
                        self.work.push(after);
                    }
                    Some(pivot)
                } else {
                    if after.len() > 0 {
                        self.work.push(after);
                    }
                    self.work.push(vec![pivot]);
                    self.work.push(before);
                    match self.next() {
                        Some(res) => Some(res),
                        None      => None
                    }
                }
            },
            None => None
        }
    }
}

// TESTS

#[test]
fn sorted_test() {
    let expected: Vec<uint> = vec![1u, 1, 1, 3, 4, 6, 7, 9, 22];
    let before: Vec<uint> = vec![9u, 7, 1, 1, 6, 3, 1, 4, 22];
    let after: Vec<uint> = before.iter().sorted().map(|x| *x).collect();

    assert_eq!(expected, after);
}

#[test]
fn sorted_by_test() {
    struct TC<'a> {
        a: f64,
        b: &'a str,
    }

    let expected: Vec<&str> = vec!["ZZZ", "ABC"];
    let before: Vec<TC> = vec![TC{a: 1.0, b: "ABC"}, TC{a: 0.75, b: "ZZZ"}];
    let after: Vec<&str> = before.iter()
        .sorted_by(|a, b| a.a.partial_cmp(&b.a).unwrap()).map(|x| x.b).collect();

    assert_eq!(expected, after);
}

// BENCHMARKS

static VEC_SIZE: int = 50000;
static PICK_SIZE: uint = 25;

#[bench]
fn standard_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0i, VEC_SIZE).map(|i| rng.gen_range(0u, 100000u)).collect();

    b.iter(|| {
        let mut numbers = numbers_raw.clone();
        numbers.sort();
        let result: Vec<&uint> = numbers.iter().take(PICK_SIZE).collect();
    });
}

#[bench]
fn lazy_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0i, VEC_SIZE).map(|i| rng.gen_range(0u, 100000u)).collect();

    b.iter(|| {
        let numbers = numbers_raw.clone();

        let result: Vec<&uint> = numbers.iter().sorted().take(PICK_SIZE).collect();
    });
}
