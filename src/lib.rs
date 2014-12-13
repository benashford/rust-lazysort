/*
 * Copyright 2014 Ben Ashford
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

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
            Some(next_work) => {
                let mut work = next_work;
                loop {
                    let pivot = work.pop().unwrap();
                    let (before, after) = work.partition(|v| (self.by)(v, &pivot) == Less);
                    if before.len() == 0 {
                        if after.len() > 0 {
                            self.work.push(after);
                        }
                        return Some(pivot)
                    } else {
                        if after.len() > 0 {
                            self.work.push(after);
                        }
                        self.work.push(vec![pivot]);
                        work = before;
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

static RANGE: uint = 1000000;
static VEC_SIZE: uint = 50000;
static PICK_SIZE_A: uint = 1000;
static PICK_SIZE_B: uint = 10000;
static PICK_SIZE_C: uint = *&VEC_SIZE;

#[bench]
fn a_standard_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|i| rng.gen_range(0u, RANGE)).collect();

    b.iter(|| {
        let mut numbers = numbers_raw.clone();
        numbers.sort();
        let result: Vec<&uint> = numbers.iter().take(PICK_SIZE_A).collect();
    });
}

#[bench]
fn a_lazy_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|i| rng.gen_range(0u, RANGE)).collect();

    b.iter(|| {
        let numbers = numbers_raw.clone();

        let result: Vec<&uint> = numbers.iter().sorted().take(PICK_SIZE_A).collect();
    });
}
#[bench]
fn b_standard_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|i| rng.gen_range(0u, RANGE)).collect();

    b.iter(|| {
        let mut numbers = numbers_raw.clone();
        numbers.sort();
        let result: Vec<&uint> = numbers.iter().take(PICK_SIZE_B).collect();
    });
}

#[bench]
fn b_lazy_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|i| rng.gen_range(0u, RANGE)).collect();

    b.iter(|| {
        let numbers = numbers_raw.clone();

        let result: Vec<&uint> = numbers.iter().sorted().take(PICK_SIZE_B).collect();
    });
}
#[bench]
fn c_standard_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|i| rng.gen_range(0u, RANGE)).collect();

    b.iter(|| {
        let mut numbers = numbers_raw.clone();
        numbers.sort();
        let result: Vec<&uint> = numbers.iter().take(PICK_SIZE_C).collect();
    });
}

#[bench]
fn c_lazy_bench(b: &mut Bencher) {
    let mut rng = task_rng();
    let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|i| rng.gen_range(0u, RANGE)).collect();

    b.iter(|| {
        let numbers = numbers_raw.clone();

        let result: Vec<&uint> = numbers.iter().sorted().take(PICK_SIZE_C).collect();
    });
}
