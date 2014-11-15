#![crate_type = "lib"]
#![crate_name = "lazysort"]

extern crate test;

use std::rand::{task_rng, Rng};

use test::Bencher;

struct LazySortIterator<O: Ord> {
    work: Vec<Vec<O>>
}

trait Sorted<O: Ord> {
    fn sorted(&mut self) -> LazySortIterator<O>;
}

impl <O: Ord, I: Iterator<O>> Sorted<O> for I {
    fn sorted(&mut self) -> LazySortIterator<O> {
        LazySortIterator {
            work: vec![self.collect()]
        }
    }
}

impl <O: Ord> Iterator<O> for LazySortIterator<O> {
    fn next(&mut self) -> Option<O> {
        match self.work.pop() {
            Some(mut next_work) => {
                let pivot = next_work.remove(0).unwrap();
                let (before, after) = next_work.partition(|v| v.cmp(&pivot) == Less);
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
