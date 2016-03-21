/*
 * Copyright 2016 rust-lazysort developers
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

#![crate_type = "lib"]
#![crate_name = "lazysort"]

#![cfg_attr(feature = "nightly", feature(test))]

use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Equal, Greater};

fn pivot(lower: usize, upper: usize) -> usize {
    return upper + ((lower - upper) / 2);
}

pub struct LazySortIterator<T, F> {
    data: Vec<T>,
    work: Vec<(usize, usize)>,
    by: F,
}

impl<T, F> LazySortIterator<T, F> where
    F: FnMut(&T, &T) -> Ordering,
{
    fn new(data: Vec<T>, by: F) -> Self where
        F: FnMut(&T, &T) -> Ordering
    {
        let l = data.len();
        LazySortIterator {
            data: data,
            work: if l == 0 {
                vec![]
            } else {
                vec![(l - 1, 0)]
            },
            by: by
        }
    }

    fn partition(&mut self, lower: usize, upper: usize, p: usize) -> usize {
        assert!(lower >= upper);
        assert!(p <= lower);
        assert!(p >= upper);

        let length = lower - upper;
        if length == 0 {
            p
        } else {
            let lasti = lower;
            let (mut i, mut nextp) = (upper, upper);
            self.data.swap(lasti, p);
            while i < lasti {
                match (self.by)(&self.data[i], &self.data[lasti]) {
                    Greater => {
                        if i != nextp {
                            self.data.swap(i, nextp);
                        }
                        nextp = nextp + 1;
                    },
                    Equal => (),
                    Less => ()
                }
                i = i + 1;
            }
            self.data.swap(nextp, lasti);
            nextp
        }
    }

    fn qsort(&mut self, lower: usize, upper: usize) -> T {
        if lower == upper {
            assert!(lower == self.data.len() - 1);
            return self.data.pop().expect("Non empty vector");
        }

        let p = pivot(lower, upper);
        let p = self.partition(lower, upper, p);

        if p == lower {
            self.work.push((p - 1, upper));
            self.qsort(lower, p)
        } else {
            self.work.push((p, upper));
            self.qsort(lower, p + 1)
        }
    }
}

pub trait Sorted {
    type Item: Ord;

    fn sorted(self) ->
        LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedPartial {
    type Item: PartialOrd;

    fn sorted_partial(self, first: bool) ->
        LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedBy {
    type Item;

    fn sorted_by<F>(self, F) -> LazySortIterator<Self::Item, F>
        where F: Fn(&Self::Item, &Self::Item) -> Ordering;
}

impl<T, I> Sorted for I where
    T: Eq + Ord,
    I: Iterator<Item=T>
{
    type Item = T;

    fn sorted(self) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        LazySortIterator::new(self.collect(), Ord::cmp)
    }
}

fn partial_cmp_first<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    match a.partial_cmp(b) {
        Some(order) => order,
        None        => Less
    }
}

fn partial_cmp_last<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    match a.partial_cmp(b) {
        Some(order) => order,
        None        => Greater
    }
}

impl<T, I> SortedPartial for I where
    T: PartialOrd,
    I: Iterator<Item=T>
{
    type Item = T;

    fn sorted_partial(self, first: bool) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        if first {
            LazySortIterator::new(self.collect(), partial_cmp_first)
        } else {
            LazySortIterator::new(self.collect(), partial_cmp_last)
        }
    }
}

impl<T, I> SortedBy for I where
    I: Iterator<Item=T>,
{
    type Item = T;

    fn sorted_by<F>(self, by: F) -> LazySortIterator<T, F> where
        F: Fn(&T, &T) -> Ordering
    {
        LazySortIterator::new(self.collect(), by)
    }
}

impl<T, F> Iterator for LazySortIterator<T, F> where
    F: FnMut(&T, &T) -> Ordering,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.work.pop() {
            Some(next_work) => {
                let (lower, upper) = next_work;
                Some(self.qsort(lower, upper))
            },
            None => None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.data.len();
        (l, Some(l))
    }
}

// TESTS

#[cfg(test)]
mod tests {
    extern crate rand;

    #[cfg(feature="nightly")]
    extern crate test;

    #[cfg(feature="nightly")]
    use self::test::Bencher;

    use self::rand::distributions::{IndependentSample, Range};

    use super::Sorted;
    use super::SortedPartial;
    use super::SortedBy;

    use std::cmp::Ordering::Equal;

    #[test]
    fn sorted_test() {
        let expected: Vec<u64> = vec![1u64, 1, 1, 3, 4, 6, 7, 9, 22];
        let before: Vec<u64> = vec![9u64, 7, 1, 1, 6, 3, 1, 4, 22];
        let after: Vec<u64> = before.iter().sorted().map(|x| *x).collect();

        assert_eq!(expected, after);
    }

    #[test]
    fn sorted_strings_test() {
        let expected: Vec<&str> = vec!["a", "cat", "mat", "on", "sat", "the"];
        let before: Vec<&str> = vec!["a", "cat", "sat", "on", "the", "mat"];
        let after: Vec<&str> = before.iter().sorted().map(|x| *x).collect();

        assert_eq!(expected, after);
    }

    #[test]
    fn sorted_string_length() {
        let expected: Vec<&str> = vec!["a", "on", "cat", "mat", "sat", "the"];
        let before: Vec<&str> = vec!["a", "cat", "sat", "on", "the", "mat"];
        let after: Vec<&str> = before.iter()
            .sorted_by(|a, b| {
                match a.len().cmp(&b.len()) {
                    Equal => a.cmp(b),
                    x => x
                }
            })
            .map(|x| *x)
            .collect();
        assert_eq!(expected, after);
    }

    #[test]
    fn empty_test() {
        let before: Vec<u64> = vec![];
        let after: Vec<u64> = before.iter().sorted().map(|x| *x).collect();
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
        let expected: Vec<u64> = vec![4, 1, 3, 2];
        let before: Vec<(f64, u64)> = vec![(0.2, 1),
                                           (0.9, 2),
                                           (0.4, 3),
                                           (0.1, 4)];

        let after: Vec<u64> = before.iter()
            .sorted_by(|&a, &b| {
                let (ax, _) = *a;
                let (bx, _) = *b;
                ax.partial_cmp(&bx).unwrap()
            })
            .map(|&(_, y)| y)
            .collect();

        assert_eq!(expected, after);
    }

    // BENCHMARKS

    static RANGE: u64 = 1000000;
    static VEC_SIZE: u64 = 50000;
    static PICK_SIZE_A: usize = 1000;
    static PICK_SIZE_B: usize = 10000;
    static PICK_SIZE_C: usize = *&VEC_SIZE as usize;

    #[cfg(feature="nightly")]
    #[bench]
    fn a_standard_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&u64> = numbers.iter().take(PICK_SIZE_A).collect();
        });
    }

    #[cfg(feature="nightly")]
    #[bench]
    fn a_lazy_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&u64> = numbers.iter().sorted().take(PICK_SIZE_A).collect();
        });
    }

    #[cfg(feature="nightly")]
    #[bench]
    fn b_standard_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&u64> = numbers.iter().take(PICK_SIZE_B).collect();
        });
    }

    #[cfg(feature="nightly")]
    #[bench]
    fn b_lazy_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&u64> = numbers.iter().sorted().take(PICK_SIZE_B).collect();
        });
    }

    #[cfg(feature="nightly")]
    #[bench]
    fn c_standard_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&u64> = numbers.iter().take(PICK_SIZE_C).collect();
        });
    }

    #[cfg(feature="nightly")]
    #[bench]
    fn c_lazy_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&u64> = numbers.iter().sorted().take(PICK_SIZE_C).collect();
        });
    }
}
