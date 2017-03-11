# Lazysort

[![Build Status](https://travis-ci.org/benashford/rust-lazysort.svg)](https://travis-ci.org/benashford/rust-lazysort)
[![](http://meritbadge.herokuapp.com/lazysort)](https://crates.io/crates/lazysort)

Adds a method to iterators that returns a sorted iterator over the data.  The sorting is achieved lazily using a quicksort algorithm.

Available via [crates.io](https://crates.io/crates/lazysort).

## Usage

```rust
extern crate lazysort;

use lazysort::Sorted;

use lazysort::SortedBy;

use lazysort::SortedPartial;
```

The `Sorted` trait adds a method `sorted` to all `Iterator<T: Ord>` which returns an iterator over the same data in default order.

The `SortedBy` trait adds a method `sorted_by` to all `Iterator<T>` which returns an iterator over the same data ordered according to the provided closure of type `|T, T| -> Ordering`

The `SortedPartial` trait adds a method `sorted_partial` to all `Iterator<T: PartialOrd>` which returns an iterator over the same data in the default order.  The method takes a parameter `first` which decides whether non-comparable values should be first or last in the results.

For example:

```rust
let data: Vec<uint> = vec![9, 1, 3, 4, 4, 2, 4];
for x in data.iter().sorted() {
	println!("{}", x);
}
```

Will print: 1, 2, 3, 4, 4, 4, 9

A more complex example.  Sort strings by length, then in default string order:

```rust
let before: Vec<&str> = vec!["a", "cat", "sat", "on", "the", "mat"];
before.iter().sorted_by(|a, b| {
    match a.len().cmp(&b.len()) {
        Equal => a.cmp(b),
        x => x
    }
})
```

This returns an iterator which yields: `a`, `on`, `cat`, `mat`, `sat`, `the`.

## Implementation details and performance

The algorithm is essentially the same as described in my blog post [using a lazy sort as an example of Clojure's lazy sequences](http://benashford.github.io/blog/2014/03/22/the-power-of-lazy-sequences/).  But made to fit in with Rust's iterators.

The full sequence from the parent iterator is read, then each call to `next` returns the next value in the sorted sequence.  The sort is done element-by-element so the full order is only realised by iterating all the way through to the end.

Previous versions did not use an in-place sort, in keeping as they were with Clojure's immutable data-structures.  The latest version however does sort in-place.

To summarise, the algorithm is the classic quicksort, but essentially depth-first; upon each call to `next` it does the work necessary to find the next item then pauses the state until the next call to `next`.

To test performance we compare it against sorting the full vector, using the `sort` function from the standard library, and also against `std::collections::BinaryHeap`.

Because of the overhead to track state, using this approach to sort a full vector is slower.  `BinaryHeap` is also slower than sorting the full vector, but faster than lazysort.

```
test benches::c_heap_bench     ... bench:   3,714,095 ns/iter (+/- 488,354)
test benches::c_lazy_bench     ... bench:   5,739,030 ns/iter (+/- 671,931)
test benches::c_standard_bench ... bench:   2,433,325 ns/iter (+/- 292,018)
```

These benchmarks are for sorting 50,000 random `uint`s in the range 0 <= x < 1000000.  Run `cargo bench` to run them.

So what's the point of lazy sorting?  As per the linked blog post, they're useful when you do not need or intend to need every value; for example you may only need the first 1,000 ordered values from a larger set.

Comparing the lazy approach `data.iter().sorted().take(x)` vs a standard approach of sorting a vector then taking the first `x` values gives the following.

The first 1,000 out of 50,000:

```
test benches::a_heap_bench     ... bench:     371,371 ns/iter (+/- 149,020)
test benches::a_lazy_bench     ... bench:     662,700 ns/iter (+/- 251,826)
test benches::a_standard_bench ... bench:   2,352,119 ns/iter (+/- 247,587)
```

The lazy approach is quite a bit faster; this is due to the 50,000 only being sorted enough to identify the first 1,000, the rest remain unsorted.  But `BinaryHeap` is the fastest option.

The first 10,000 out of 50,000:

```
test benches::b_heap_bench     ... bench:   1,133,117 ns/iter (+/- 115,459)
test benches::b_lazy_bench     ... bench:   1,725,260 ns/iter (+/- 332,242)
test benches::b_standard_bench ... bench:   2,454,311 ns/iter (+/- 265,727)
```

The lazy approach is still faster, slightly.  And `BinaryHeap` remains the fastest overall.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
