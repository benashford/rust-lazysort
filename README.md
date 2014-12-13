# Lazysort

[![Build Status](https://travis-ci.org/benashford/rust-lazysort.svg)](https://travis-ci.org/benashford/rust-lazysort)

Adds a method to iterators that returns a sorted iterator over the data.  The sorting is achieved lazily using a quicksort algorithm.

Available via [crates.io](https://crates.io/crates/lazysort).

Requires a recent Rust 0.13 nightly build.

## Usage

```rust
extern crate lazysort;

use lazysort::Sorted;

use lazysort::SortedBy;
```

The `Sorted` trait adds a method `sorted` to all `Iterator<T: Ord>` which returns an iterator over the same data in default order.

The `SortedBy` trait adds a method `sorted_by` to all `Iterator<T>` which returns an iterator over the same data ordered according to the provided closure of type `|T, T| -> Ordering`

For example:

```rust
let data: Vec<uint> = vec![9, 1, 3, 4, 4, 2, 4];
for x in data.iter().sorted() {
	println!("{}", x);
}
```

Will print: 1, 2, 3, 4, 4, 4, 9

## Implementation details and performance

The algorithm is essentially the same as described in my blog post [using a lazy sort as an example of Clojure's lazy sequences](http://benashford.github.io/blog/2014/03/22/the-power-of-lazy-sequences/).  But made to fit in with Rust's iterators.

The full sequence from the parent iterator is read, then each call to `next` returns the next value in the sorted sequence.  The sort is done element-by-element so the full order is only realised by iterating all the way through to the end.

Previous versions did not use an in-place sort, in keeping as they were with Clojure's immutable data-structures.  The latest version however does sort in-place.

To summarise, the algorithm is the classic quicksort, but essentially depth-first; upon each call to `next` it does the work necessary to find the next item then pauses the state until the next call to `next`.

Because of the overhead to track state, using this approach to sort a full vector is slightly slower than the `sort` function from the standard library:

```
test c_lazy_bench     ... bench:   8002794 ns/iter (+/- 1592998)
test c_standard_bench ... bench:   6819930 ns/iter (+/- 1154443)
```

These benchmarks are for sorting 50,000 random `uint`s in the range 0 <= x < 1000000.  Run `cargo bench` to run them.

So what's the point of lazy sorting?  As per the linked blog post, they're useful when you do not need or intend to need every value; for example you may only need the first 1,000 ordered values from a larger set.

Comparing the lazy approach `data.iter().sorted().take(x)` vs a standard approach of sorting a vector then taking the first `x` values gives the following.

The first 1,000 out of 50,000:

```
test a_lazy_bench     ... bench:    899866 ns/iter (+/- 691194)
test a_standard_bench ... bench:   6751776 ns/iter (+/- 983935)
```

The lazy approach is significantly faster.

The first 10,000 out of 50,000:

```
test b_lazy_bench     ... bench:   2309792 ns/iter (+/- 596399)
test b_standard_bench ... bench:   6796466 ns/iter (+/- 819747)
```

The lazy approach is still faster.

These results change on a regular basis as Rust is a fast-moving target, run `cargo bench` to see for yourself the latest numbers.

## License 

```
   Copyright 2014 Ben Ashford

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```
