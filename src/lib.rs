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
use std::cmp::Ordering::{Greater, Less};

fn pivot(lower: usize, upper: usize) -> usize {
    return upper + ((lower - upper) / 2);
}

#[inline(always)]
unsafe fn cmp_by<F, T>(by: &F, data: &mut [T], a: usize, b: usize) -> Ordering
where
    F: Fn(&T, &T) -> Ordering,
{
    by(data.get_unchecked(a), data.get_unchecked(b))
}

pub struct LazySortIterator<T, F> {
    data: Vec<T>,
    work: Vec<(usize, usize)>,
    by: F,
}

impl<T, F> LazySortIterator<T, F>
where
    F: Fn(&T, &T) -> Ordering,
{
    fn new(data: Vec<T>, by: F) -> Self {
        let l = data.len();
        let mut work = Vec::with_capacity(l / 4);
        if l > 0 {
            work.push((l - 1, 0));
        }
        LazySortIterator {
            data: data,
            work: work,
            by: by,
        }
    }

    fn partition(&mut self, lower: usize, upper: usize, p: usize) -> usize {
        // To make things more fun - well there is a real reason, which is that we can
        // simply `pop` values to remove the lowest value - the lower values are stored
        // at the higher indexes.  So in this function `lower` will actually be higher
        // than `upper`

        unsafe {
            let mut i = upper;
            let mut nextp = upper;

            self.data.swap(lower, p);

            while i < lower {
                if cmp_by(&self.by, &mut self.data, i, lower) == Greater {
                    if i != nextp {
                        self.data.swap(i, nextp);
                    }
                    nextp += 1;
                }
                i += 1;
            }

            self.data.swap(nextp, lower);
            nextp
        }
    }

    fn qsort(&mut self, lower: usize, upper: usize) -> T {
        // If lower and upper are the same, then just pop the next value
        // If lower and upper are adjacent, then manually swap depending on ordering
        // everything else, do the next stage of a quick sort
        match lower - upper {
            0 => self.data.pop().expect("Non empty vector"),
            1 => unsafe {
                if cmp_by(&self.by, &mut self.data, lower, upper) == Greater {
                    self.data.swap(lower, upper);
                }
                self.work.push((upper, upper));
                self.data.pop().expect("Non empty vector")
            },
            _ => {
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
    }
}

pub trait Sorted {
    type Item: Ord;

    fn sorted(self) -> LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedPartial {
    type Item: PartialOrd;

    fn sorted_partial(
        self,
        first: bool,
    ) -> LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedBy {
    type Item;

    fn sorted_by<F>(self, F) -> LazySortIterator<Self::Item, F>
    where
        F: Fn(&Self::Item, &Self::Item) -> Ordering;
}

impl<T, I> Sorted for I
where
    T: Eq + Ord,
    I: Iterator<Item = T>,
{
    type Item = T;

    fn sorted(self) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        LazySortIterator::new(self.collect(), Ord::cmp)
    }
}

fn partial_cmp_first<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    match a.partial_cmp(b) {
        Some(order) => order,
        None => Less,
    }
}

fn partial_cmp_last<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    match a.partial_cmp(b) {
        Some(order) => order,
        None => Greater,
    }
}

impl<T, I> SortedPartial for I
where
    T: PartialOrd,
    I: Iterator<Item = T>,
{
    type Item = T;

    fn sorted_partial(
        self,
        first: bool,
    ) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        if first {
            LazySortIterator::new(self.collect(), partial_cmp_first)
        } else {
            LazySortIterator::new(self.collect(), partial_cmp_last)
        }
    }
}

impl<T, I> SortedBy for I
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn sorted_by<F>(self, by: F) -> LazySortIterator<T, F>
    where
        F: Fn(&T, &T) -> Ordering,
    {
        LazySortIterator::new(self.collect(), by)
    }
}

impl<T, F> Iterator for LazySortIterator<T, F>
where
    F: Fn(&T, &T) -> Ordering,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.work.pop() {
            Some(next_work) => {
                let (lower, upper) = next_work;
                Some(self.qsort(lower, upper))
            }
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.data.len();
        (l, Some(l))
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::Sorted;
    use super::SortedPartial;
    use super::SortedBy;

    use std::cmp::Ordering::Equal;

    #[test]
    fn single_test() {
        let expected: Vec<u64> = vec![1];
        let before: Vec<u64> = vec![1];
        let after: Vec<u64> = before.into_iter().sorted().collect();

        assert_eq!(expected, after);
    }

    #[test]
    fn pair_test() {
        let expected: Vec<u64> = vec![1, 2];
        let before_ordered: Vec<u64> = vec![1, 2];
        let before_unordered: Vec<u64> = vec![2, 1];

        let after_ordered: Vec<u64> = before_ordered.into_iter().sorted().collect();
        assert_eq!(expected, after_ordered);

        let after_unordered: Vec<u64> = before_unordered.into_iter().sorted().collect();
        assert_eq!(expected, after_unordered);
    }

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
        let after: Vec<&str> = before
            .iter()
            .sorted_by(|a, b| match a.len().cmp(&b.len()) {
                Equal => a.cmp(b),
                x => x,
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
        let before: Vec<(f64, u64)> = vec![(0.2, 1), (0.9, 2), (0.4, 3), (0.1, 4)];

        let after: Vec<u64> = before
            .iter()
            .sorted_by(|&a, &b| {
                let (ax, _) = *a;
                let (bx, _) = *b;
                ax.partial_cmp(&bx).unwrap()
            })
            .map(|&(_, y)| y)
            .collect();

        assert_eq!(expected, after);
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    extern crate rand;

    extern crate test;

    use self::test::{black_box, Bencher};

    use self::rand::distributions::{IndependentSample, Range};

    use super::Sorted;

    use std::cmp::Ordering;
    use std::collections::BinaryHeap;
    use std::iter::FromIterator;

    static RANGE: u64 = 1000000;
    static VEC_SIZE: u64 = 50000;
    static PICK_SIZE_A: usize = 1000;
    static PICK_SIZE_B: usize = 10000;
    static PICK_SIZE_C: usize = 50000;

    fn data() -> Vec<u64> {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        (0u64..VEC_SIZE)
            .map(|_| between.ind_sample(&mut rng))
            .collect()
    }

    #[bench]
    fn a_standard_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let mut numbers = black_box(&input).clone();
            numbers.sort();
            let pick: Vec<u64> = numbers.into_iter().take(PICK_SIZE_A).collect();
            black_box(pick)
        });
    }

    #[bench]
    fn a_lazy_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let numbers = black_box(&input).clone();

            let pick: Vec<u64> = numbers.into_iter().sorted().take(PICK_SIZE_A).collect();
            black_box(pick)
        });
    }

    #[bench]
    fn a_heap_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let mut heap = BinaryHeap::from_iter(black_box(&input).iter().cloned().map(RevOrd));

            let mut pick: Vec<u64> = Vec::with_capacity(PICK_SIZE_A);
            for _ in 0..PICK_SIZE_A {
                pick.push(heap.pop().unwrap().0);
            }
            black_box(pick)
        });
    }

    #[bench]
    fn b_standard_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let mut numbers = black_box(&input).clone();
            numbers.sort();
            let pick: Vec<u64> = numbers.into_iter().take(PICK_SIZE_B).collect();
            black_box(pick)
        });
    }

    #[bench]
    fn b_lazy_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let numbers = black_box(&input).clone();

            let pick: Vec<u64> = numbers.into_iter().sorted().take(PICK_SIZE_B).collect();
            black_box(pick)
        });
    }

    #[bench]
    fn b_heap_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let mut heap = BinaryHeap::from_iter(black_box(&input).iter().cloned().map(RevOrd));

            let mut pick: Vec<u64> = Vec::with_capacity(PICK_SIZE_B);
            for _ in 0..PICK_SIZE_B {
                pick.push(heap.pop().unwrap().0);
            }
            black_box(pick)
        });
    }

    #[bench]
    fn c_standard_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let mut numbers = black_box(&input).clone();
            numbers.sort();
            let pick: Vec<u64> = numbers.into_iter().take(PICK_SIZE_C).collect();
            black_box(pick)
        });
    }

    #[bench]
    fn c_lazy_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let numbers = black_box(&input).clone();

            let pick: Vec<u64> = numbers.into_iter().sorted().take(PICK_SIZE_C).collect();
            black_box(pick)
        });
    }

    #[bench]
    fn c_heap_bench(b: &mut Bencher) {
        let input = data();

        b.iter(|| {
            let mut heap = BinaryHeap::from_iter(black_box(&input).iter().cloned().map(RevOrd));

            let mut pick: Vec<u64> = Vec::with_capacity(PICK_SIZE_C);
            for _ in 0..PICK_SIZE_C {
                pick.push(heap.pop().unwrap().0);
            }
            black_box(pick)
        });
    }

    // BinaryHeap is a max heap. We want to extract the minimum values so
    // reverse the ordering.
    struct RevOrd<V>(V);

    impl<V> PartialOrd for RevOrd<V>
    where
        V: PartialOrd,
    {
        fn partial_cmp(&self, other: &RevOrd<V>) -> Option<Ordering> {
            other.0.partial_cmp(&self.0)
        }
    }

    impl<V> Ord for RevOrd<V>
    where
        V: Ord,
    {
        fn cmp(&self, other: &RevOrd<V>) -> Ordering {
            other.0.cmp(&self.0)
        }
    }

    impl<V> PartialEq for RevOrd<V>
    where
        V: PartialEq,
    {
        fn eq(&self, other: &RevOrd<V>) -> bool {
            other.0.eq(&self.0)
        }
    }

    impl<V> Eq for RevOrd<V>
    where
        V: Eq,
    {
    }
}
