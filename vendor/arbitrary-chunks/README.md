# arbitrary-chunks

![Crates.io](https://img.shields.io/crates/l/arbitrary-chunks?style=flat-square)
![Crates.io](https://img.shields.io/crates/v/arbitrary-chunks?style=flat-square)

An iterator that allows specifying an input array of arbitrary chunk-sizes with which to split a vector or array. As with the standard `.chunks()`, this iterator also includes `_mut()`, `_exact()` and `_exact_mut()` variants.

## Usage

By default, this iterator is implemented for `[T]`, meaning it works for both arrays and Vec of any type.

If there is not enough data to satisfy the provided chunk length, you will get all the remaining data for that chunk, so it will be shorter than expected. To instead stop early if there is not enough data, see the `.arbitrary_chunks_exact()` variant.

```rust
use arbitrary_chunks::ArbitraryChunks;

let chunks: Vec<usize> = vec![1, 3, 1];
let data: Vec<i32> = vec![0, 1, 2, 3, 4];

let chunked_data: Vec<Vec<i32>> = data
    .arbitrary_chunks(&chunks)
    .map(|chunk| chunk.to_vec())
    .collect();

assert_eq!(vec![0], chunked_data[0]);
assert_eq!(vec![1, 2, 3], chunked_data[1]);
assert_eq!(vec![4], chunked_data[2]);
```

### Exact Variant

Unlike the regular variant, the exact variant's iterator will end early if there is not enough data to satisfy the chunk. Instead, you will be able to access the remainder of the data with `.remainder()`.

```rust
use arbitrary_chunks::ArbitraryChunks;

let chunks: Vec<usize> = vec![1, 3];
let data: Vec<i32> = vec![0, 1, 2];
let mut iter = data.arbitrary_chunks_exact(&chunks);

assert_eq!(vec![0], iter.next().unwrap());
assert_eq!(None, iter.next());
assert_eq!(vec![1, 2], iter.remainder());
```

### Mutable Variants

Each of the regular and exact variants also have their own mutable variants. These allow you to mutably modify slices and vectors in arbitrarily-sized chunks.

```rust
use arbitrary_chunks::ArbitraryChunks;

let chunks: Vec<usize> = vec![1, 3, 1];
let data: Vec<i32> = vec![0, 1, 2, 3, 4];

let iter_1 = data.arbitrary_chunks_mut(&chunks);
let iter_2 = data.arbitrary_chunks_exact_mut(&chunks);
```

### Parallel Iterator

This can be used in parallel with rayon using `.par_bridge()`, for example:

```rust
use arbitrary_chunks::ArbitraryChunks;
use rayon::prelude::*;

let chunks: Vec<usize> = vec![1, 3, 1];
let data: Vec<i32> = vec![0, 1, 2, 3, 4];

data
    .arbitrary_chunks(&chunks)
    .par_bridge()
    .for_each(|chunk| {
        assert!(chunk.len() >= 1 && chunk.len() <= 3);
        println!("{:?}", chunk);
    });

// Prints (in pseudo-random order):
// [1, 2, 3]
// [0]
// [4]
```

## Motivation

This library was inspired by the need to mutably modify many sections of the same slice concurrently. With this library plus Rayon, you are able to mutably borrow and modify slices safely from many threads at the same time, without upsetting the borrow checker.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
