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

use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Greater};

fn pivot(lower: uint, upper: uint) -> uint {
    return lower + ((upper - lower) / 2);
}

pub struct LazySortIterator<'a, T: Clone + 'a> {
    data: Vec<T>,
    work: Vec<(uint, uint)>,
    by: Box<Fn(&T, &T) -> Ordering + 'a>,
}

impl <'a, T: Clone> LazySortIterator<'a, T> {
    fn new(data: Vec<T>, by: Box<Fn(&T, &T) -> Ordering + 'a>) -> LazySortIterator<'a, T> {
        let l = data.len();
        LazySortIterator {
            data: data,
            work: if l == 0 {
                vec![]
            } else {
                vec![(0, l - 1)]
            },
            by: by
        }
    }

    fn partition(&mut self, lower: uint, upper: uint, p: uint) -> uint {
        assert!(lower <= upper);
        assert!(p >= lower);
        assert!(p <= upper);

        let length = upper - lower;
        if length == 0 {
            p
        } else {
            let lasti = upper;
            let (mut i, mut nextp) = (lower, lower);
            self.data.swap(lasti, p);
            while i < lasti {
                if (self.by)(&self.data[i], &self.data[lasti]) == Less {
                    self.data.swap(i, nextp);
                    nextp = nextp + 1;
                }
                i = i + 1;
            }
            self.data.swap(nextp, lasti);
            nextp
        }
    }

    fn qsort(&mut self, lower: uint, upper: uint) -> T {
        assert!(lower <= upper);

        if lower == upper {
            return self.data[lower].clone();
        }

        let p = pivot(lower, upper);
        let p = self.partition(lower, upper, p);

        if p < upper {
            self.work.push((p + 1, upper));
        }
        self.qsort(lower, p)
    }
}

pub trait Sorted<'a, O: Ord + Clone> {
    fn sorted(self) -> LazySortIterator<'a, O>;
}

pub trait SortedPartial<'a, O: PartialOrd + Clone> {
    fn sorted_partial(self, first: bool) -> LazySortIterator<'a, O>;
}

pub trait SortedBy<'a, T: Clone> {
    fn sorted_by(self, Box<Fn(&T, &T) -> Ordering + 'a>) -> LazySortIterator<'a, T>;
}

impl <'a, O: Ord + Clone, I: Iterator<Item=O>> Sorted<'a, O> for I {
    fn sorted(self) -> LazySortIterator<'a, O> {
        let by = box |&: a: &O, b: &O| -> Ordering { a.cmp(b) };
        LazySortIterator::new(self.collect(), by)
    }
}

impl <'a, O: PartialOrd + Clone, I: Iterator<Item=O>> SortedPartial<'a, O> for I {
    fn sorted_partial(self, first: bool) -> LazySortIterator<'a, O> {
        let by = box move |&: a: &O, b: &O| {
            match a.partial_cmp(b) {
                Some(order) => order,
                None => if first {
                    Less
                } else {
                    Greater
                }
            }
        };
        LazySortIterator::new(self.collect(), by)
    }
}

impl <'a, T: Clone, I: Iterator<Item=T>> SortedBy<'a, T> for I {
    fn sorted_by(self, by: Box<Fn(&T, &T) -> Ordering + 'a>) -> LazySortIterator<'a, T> {
        LazySortIterator::new(self.collect(), by)
    }
}

impl <'a, T: Clone> Iterator for LazySortIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.work.pop() {
            Some(next_work) => {
                let (lower, upper) = next_work;
                Some(self.qsort(lower, upper))
            },
            None => None
        }
    }
}

// TESTS

#[cfg(test)]
mod tests {
    use std::rand::{thread_rng, Rng};
    use test::Bencher;

    use super::Sorted;
    use super::SortedPartial;
    use super::SortedBy;

    #[test]
    fn sorted_test() {
        let expected: Vec<uint> = vec![1u, 1, 1, 3, 4, 6, 7, 9, 22];
        let before: Vec<uint> = vec![9u, 7, 1, 1, 6, 3, 1, 4, 22];
        let after: Vec<uint> = before.iter().sorted().map(|x| *x).collect();

        println!("AFTER {}", after);
        assert_eq!(expected, after);
    }

    #[test]
    fn empty_test() {
        let before: Vec<uint> = vec![];
        let after: Vec<uint> = before.iter().sorted().map(|x| *x).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn sorted_partial_test() {
        let expected: Vec<f64> = vec![0.9_f64, 1.0, 1.0, 1.1, 75.3, 75.3];
        let before: Vec<f64> = vec![1.0_f64, 1.1, 0.9, 75.3, 1.0, 75.3];
        let after: Vec<f64> = before.iter().sorted_partial(true).map(|x| *x).collect();

        assert_eq!(expected, after);
    }

    #[test]
    fn sorted_by_test() {
        struct TC<'a> {
            a: f64,
            b: &'a str
        }

        let expected: Vec<&str> = vec!["ZZZ", "ABC"];
        let before: Vec<TC> = vec![TC{a: 1.0, b: "ABC"},
                                   TC{a: 0.75, b: "ZZZ"}];
        let after: Vec<&str> = before.iter()
            .sorted_by(box |a, b| a.a.partial_cmp(&b.a).unwrap()).map(|x| x.b).collect();

        println!("AFTER {}", after);
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
        let mut rng = thread_rng();
        let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|_| rng.gen_range(0u, RANGE)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&uint> = numbers.iter().take(PICK_SIZE_A).collect();
        });
    }

    #[bench]
    fn a_lazy_bench(b: &mut Bencher) {
        let mut rng = thread_rng();
        let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|_| rng.gen_range(0u, RANGE)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&uint> = numbers.iter().sorted().take(PICK_SIZE_A).collect();
        });
    }
    #[bench]
    fn b_standard_bench(b: &mut Bencher) {
        let mut rng = thread_rng();
        let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|_| rng.gen_range(0u, RANGE)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&uint> = numbers.iter().take(PICK_SIZE_B).collect();
        });
    }

    #[bench]
    fn b_lazy_bench(b: &mut Bencher) {
        let mut rng = thread_rng();
        let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|_| rng.gen_range(0u, RANGE)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&uint> = numbers.iter().sorted().take(PICK_SIZE_B).collect();
        });
    }
    #[bench]
    fn c_standard_bench(b: &mut Bencher) {
        let mut rng = thread_rng();
        let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|_| rng.gen_range(0u, RANGE)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&uint> = numbers.iter().take(PICK_SIZE_C).collect();
        });
    }

    #[bench]
    fn c_lazy_bench(b: &mut Bencher) {
        let mut rng = thread_rng();
        let numbers_raw: Vec<uint> = range(0u, VEC_SIZE).map(|_| rng.gen_range(0u, RANGE)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&uint> = numbers.iter().sorted().take(PICK_SIZE_C).collect();
        });
    }
}
