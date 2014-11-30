# Lazysort

Adds a method to iterators that returns a sorted iterator over the data.  The sorting is achieved lazily using a quicksort algorithm.

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

The algorithm is essentially the same as described in my blog post [using a lazy sort as an example of Clojure's lazy sequences](http://benashford.github.io/blog/2014/03/22/the-power-of-lazy-sequences/).

The full sequence from the parent iterator is read, then each call to `next` returns the next value in the sorted sequence.  The sort is done element-by-element so the full order is only realised by iterating all the way through to the end.

This is not an in-place sort, the original vector (created from the original iterator) is broken down into smaller-and-smaller chunks on each call to `next`.  As such, using this algorithm to fully sort a vector will be (much) slower than using the built in sort:

```
test c_lazy_bench     ... bench:  14183181 ns/iter (+/- 2302170)
test c_standard_bench ... bench:   6355806 ns/iter (+/- 990029)
```

More than twice as slow, in fact.

These benchmarks are for sorting 50,000 random `uint`s in the range 0 <= x < 1000000.  Run `cargo bench` to run them.

So what's the point of lazy sorting?  As per the linked blog post, they're useful when you do not need or intend to need every value; for example you may only need the first 1,000 ordered values from a larger set.

Comparing the lazy approach `data.iter().sorted().take(x)` vs a standard approach of sorting a vector then taking the first `x` values gives the following.

The first 1,000 out of 50,000:

```
test a_lazy_bench     ... bench:   1422118 ns/iter (+/- 1355232)
test a_standard_bench ... bench:   6374353 ns/iter (+/- 1103586)
```

The lazy approach is significantly faster.

The first 10,000 out of 50,000:

```
test b_lazy_bench     ... bench:   3844710 ns/iter (+/- 1314643)
test b_standard_bench ... bench:   6594943 ns/iter (+/- 762343)
```

The lazy approach is still faster.  But at around 20,000 out of 50,000 you may as well sort the whole dataset.

## Future work

This is a relatively naive implementation based on the Clojure version, therefore there are undoubtedly Rust-specific efficiencies to be gained. For example: the use of an in-place quicksort, lazily achieved.

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