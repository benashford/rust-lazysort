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

fn partition<F, T>(by: &F, data: &mut [T], lower: usize, upper: usize, p: usize) -> usize
where
    F: Fn(&T, &T) -> Ordering,
{
    // To make things more fun - well there is a real reason, which is that we can
    // simply `pop` values to remove the lowest value - the lower values are stored
    // at the higher indexes.  So in this function `lower` will actually be higher
    // than `upper`

    unsafe {
        let mut i = upper;
        let mut nextp = upper;

        data.swap(lower, p);

        while i < lower {
            if cmp_by(by, data, i, lower) == Greater {
                if i != nextp {
                    data.swap(i, nextp);
                }
                nextp += 1;
            }
            i += 1;
        }

        data.swap(nextp, lower);
        nextp
    }
}

fn qsort<F, T>(
    by: &F,
    data: &mut Vec<T>,
    work: &mut Vec<(usize, usize)>,
    lower: usize,
    upper: usize,
) -> T
where
    F: Fn(&T, &T) -> Ordering,
{
    // If lower and upper are the same, then just pop the next value
    // If lower and upper are adjacent, then manually swap depending on ordering
    // everything else, do the next stage of a quick sort
    match lower - upper {
        0 => data.pop().expect("Non empty vector"),
        1 => unsafe {
            if cmp_by(by, data, lower, upper) == Greater {
                data.swap(lower, upper);
            }
            work.push((upper, upper));
            data.pop().expect("Non empty vector")
        },
        _ => {
            let p = pivot(lower, upper);
            let p = partition(by, data, lower, upper, p);
            if p == lower {
                work.push((p - 1, upper));
                qsort(by, data, work, lower, p)
            } else {
                work.push((p, upper));
                qsort(by, data, work, lower, p + 1)
            }
        }
    }
}

fn make_work(len: usize) -> Vec<(usize, usize)> {
    let mut work = Vec::with_capacity(len / 4);
    if len > 0 {
        work.push((len - 1, 0));
    }
    work
}

macro_rules! lazy_sort_iter_struct {
    ($name:ident) => {
        pub struct $name<T> {
            data: Vec<T>,
            work: Vec<(usize, usize)>
        }
    }
}

macro_rules! lazy_sort_iter_struct_new {
    () => {
        fn new(data: Vec<T>) -> Self {
            let work = make_work(data.len());
            Self {
                data: data,
                work: work
            }
        }
    }
}

macro_rules! lazy_sort_iter_struct_qsort {
    ($cmp_f:path) => {
        fn qsort(&mut self, lower: usize, upper: usize) -> T {
            qsort(&$cmp_f, &mut self.data, &mut self.work, lower, upper)
        }
    }
}

lazy_sort_iter_struct!(LazySortIterator);

impl<T> LazySortIterator<T>
where
    T: Ord,
{
    lazy_sort_iter_struct_new!();
    lazy_sort_iter_struct_qsort!(Ord::cmp);
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

lazy_sort_iter_struct!(LazySortIteratorPartialFirst);
lazy_sort_iter_struct!(LazySortIteratorPartialLast);

impl<T> LazySortIteratorPartialFirst<T>
where
    T: PartialOrd,
{
    lazy_sort_iter_struct_new!();
    lazy_sort_iter_struct_qsort!(partial_cmp_first);
}

impl<T> LazySortIteratorPartialLast<T>
where
    T: PartialOrd,
{
    lazy_sort_iter_struct_new!();
    lazy_sort_iter_struct_qsort!(partial_cmp_last);
}

pub struct LazySortIteratorBy<T, F> {
    data: Vec<T>,
    work: Vec<(usize, usize)>,
    by: F,
}

impl<T, F> LazySortIteratorBy<T, F>
where
    F: Fn(&T, &T) -> Ordering,
{
    fn new(data: Vec<T>, by: F) -> Self {
        let work = make_work(data.len());
        LazySortIteratorBy {
            data: data,
            work: work,
            by: by,
        }
    }

    fn qsort(&mut self, lower: usize, upper: usize) -> T {
        qsort(&self.by, &mut self.data, &mut self.work, lower, upper)
    }
}

pub trait Sorted {
    type Item: Ord;

    fn sorted(self) -> LazySortIterator<Self::Item>;
}

pub trait SortedPartial {
    type Item: PartialOrd;

    fn sorted_partial_first(self) -> LazySortIteratorPartialFirst<Self::Item>;
    fn sorted_partial_last(self) -> LazySortIteratorPartialLast<Self::Item>;
}

pub trait SortedBy {
    type Item;

    fn sorted_by<F>(self, F) -> LazySortIteratorBy<Self::Item, F>
    where
        F: Fn(&Self::Item, &Self::Item) -> Ordering;
}

impl<T, I> Sorted for I
where
    T: Eq + Ord,
    I: Iterator<Item = T>,
{
    type Item = T;

    fn sorted(self) -> LazySortIterator<T> {
        LazySortIterator::new(self.collect())
    }
}

impl<T, I> SortedPartial for I
where
    T: PartialOrd,
    I: Iterator<Item = T>,
{
    type Item = T;

    fn sorted_partial_first(self) -> LazySortIteratorPartialFirst<T> {
        LazySortIteratorPartialFirst::new(self.collect())
    }

    fn sorted_partial_last(self) -> LazySortIteratorPartialLast<T> {
        LazySortIteratorPartialLast::new(self.collect())
    }
}

impl<T, I> SortedBy for I
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn sorted_by<F>(self, by: F) -> LazySortIteratorBy<T, F>
    where
        F: Fn(&T, &T) -> Ordering,
    {
        LazySortIteratorBy::new(self.collect(), by)
    }
}

macro_rules! add_next {
    () => {
        #[inline]
        fn next(&mut self) -> Option<T> {
            match self.work.pop() {
                Some((lower, upper)) => Some(self.qsort(lower, upper)),
                None => None
            }
        }
    }
}

macro_rules! add_size_hint {
    () => {
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let l = self.data.len();
            (l, Some(l))
        }
    }
}

impl<T> Iterator for LazySortIterator<T>
where
    T: Ord,
{
    type Item = T;

    add_next!();
    add_size_hint!();
}

impl<T> Iterator for LazySortIteratorPartialFirst<T>
where
    T: PartialOrd,
{
    type Item = T;

    add_next!();
    add_size_hint!();
}

impl<T> Iterator for LazySortIteratorPartialLast<T>
where
    T: PartialOrd,
{
    type Item = T;

    add_next!();
    add_size_hint!();
}

impl<T, F> Iterator for LazySortIteratorBy<T, F>
where
    F: Fn(&T, &T) -> Ordering,
{
    type Item = T;

    add_next!();
    add_size_hint!();
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
        let after: Vec<f64> = before.iter().sorted_partial_first().map(|x| *x).collect();

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
