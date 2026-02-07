//! # arbitrary-chunks
//! An iterator that allows specifying an input array of arbitrary chunk-sizes with which to split a vector or array. As with the standard `.chunks()`, this iterator also includes `_mut()`, `_exact()` and `_exact_mut()` variants.
//!
//! ## Usage
//!
//! By default, this iterator is implemented for `[T]`, meaning it works for both arrays and Vec of any type.
//!
//! If there is not enough data to satisfy the provided chunk length, you will get all the remaining data for that chunk, so it will be shorter than expected. To instead stop early if there is not enough data, see the `.arbitrary_chunks_exact()` variant.
//!
//! ```rust
//! use arbitrary_chunks::ArbitraryChunks;
//!
//! let chunks: Vec<usize> = vec![1, 3, 1];
//! let data: Vec<i32> = vec![0, 1, 2, 3, 4];
//!
//! let chunked_data: Vec<Vec<i32>> = data
//!   .arbitrary_chunks(&chunks)
//!   .map(|chunk| chunk.to_vec())
//!   .collect();
//!
//! assert_eq!(vec![0], chunked_data[0]);
//! assert_eq!(vec![1, 2, 3], chunked_data[1]);
//! assert_eq!(vec![4], chunked_data[2]);
//! ```
//!
//! ## Exact Variant
//!
//! Unlike the regular variant, the exact variant's iterator will end early if there is not enough data to satisfy the chunk. Instead, you will be able to access the remainder of the data with `.remainder()`.
//!
//! ```rust
//! use arbitrary_chunks::ArbitraryChunks;
//!
//! let chunks: Vec<usize> = vec![1, 3];
//! let data: Vec<i32> = vec![0, 1, 2];
//! let mut iter = data.arbitrary_chunks_exact(&chunks);
//!
//! assert_eq!(vec![0], iter.next().unwrap());
//! assert_eq!(None, iter.next());
//! assert_eq!(vec![1, 2], iter.remainder());
//! ```
//!
//! ### Mutable Variants
//!
//! Each of the regular and exact variants also have their own mutable variants. These allow you to mutably modify slices and vectors in arbitrarily-sized chunks.
//!
//! ```ignore
//! use arbitrary_chunks::ArbitraryChunks;
//!
//! let chunks: Vec<usize> = vec![1, 3, 1];
//! let data: Vec<i32> = vec![0, 1, 2, 3, 4];
//!
//! let iter_1 = data.arbitrary_chunks_mut(chunks.clone());
//! let iter_2 = data.arbitrary_chunks_exact_mut(chunks);
//! ```
//!
//! ### Parallel Iterator
//!
//! This can be used in parallel with rayon using `.par_bridge()`, for example:
//!
//! ```rust
//! use arbitrary_chunks::ArbitraryChunks;
//! use rayon::prelude::*;
//!
//! let chunks: Vec<usize> = vec![1, 3, 1];
//! let data: Vec<i32> = vec![0, 1, 2, 3, 4];
//!
//! data
//!   .arbitrary_chunks(&chunks)
//!   .par_bridge()
//!   .for_each(|chunk| {
//!     assert!(chunk.len() >= 1 && chunk.len() <= 3);
//!     println!("{:?}", chunk);
//!   });
//!
//! // Prints (in pseudo-random order):
//! // [1, 2, 3]
//! // [0]
//! // [4]
//! ```
//!
//! ## Motivation
//!
//! This library was inspired by the need to mutably modify many sections of the same slice concurrently. With this library plus Rayon, you are able to mutably borrow and modify slices safely from many threads at the same time, without upsetting the borrow checker.
//!
//! ## License
//!
//! Licensed under either of
//!
//! * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//! * MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
//!

use std::cmp::min;
use std::mem;

pub struct ArbitraryChunkMut<'a, 'b, T: 'a> {
    data: &'a mut [T],
    counts: &'b [usize],
    cursor: usize,
}

impl<'a, 'b, T> Iterator for ArbitraryChunkMut<'a, 'b, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > self.counts.len() - 1 {
            return None;
        }

        let c = self.counts[self.cursor];
        self.cursor += 1;

        if c == 0 {
            return Some(&mut []);
        }

        if self.data.is_empty() {
            return None;
        }

        let point = min(c, self.data.len());
        let slice = mem::take(&mut self.data);
        let (l, r) = slice.split_at_mut(point);
        self.data = r;

        Some(l)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.counts.len() - self.cursor;
        (remaining, Some(remaining))
    }
}

pub struct ArbitraryChunk<'a, 'b, T: 'a> {
    data: &'a [T],
    counts: &'b [usize],
    cursor: usize,
}

impl<'a, 'b, T> Iterator for ArbitraryChunk<'a, 'b, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > self.counts.len() - 1 {
            return None;
        }

        let c = self.counts[self.cursor];
        self.cursor += 1;

        if c == 0 {
            return Some(&[]);
        }

        if self.data.is_empty() {
            return None;
        }

        let point = min(c, self.data.len());
        let slice = mem::take(&mut self.data);
        let (l, r) = slice.split_at(point);
        self.data = r;

        Some(l)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.counts.len() - self.cursor;
        (remaining, Some(remaining))
    }
}

pub struct ArbitraryChunkExactMut<'a, 'b, T: 'a> {
    data: &'a mut [T],
    counts: &'b [usize],
    cursor: usize,
}

impl<'a, 'b, T> Iterator for ArbitraryChunkExactMut<'a, 'b, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > self.counts.len() - 1 {
            return None;
        }

        let c = self.counts[self.cursor];
        self.cursor += 1;

        if c == 0 {
            return Some(&mut []);
        }

        if self.data.is_empty() || c > self.data.len() {
            return None;
        }

        let slice = mem::take(&mut self.data);
        let (l, r) = slice.split_at_mut(c);
        self.data = r;

        Some(l)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.counts.len() - self.cursor;
        (remaining, Some(remaining))
    }
}

impl<'a, 'b, T> ArbitraryChunkExactMut<'a, 'b, T> {
    pub fn remainder(&'a mut self) -> &'a mut [T] {
        self.data
    }
}

pub struct ArbitraryChunkExact<'a, 'b, T: 'a> {
    data: &'a [T],
    counts: &'b [usize],
    cursor: usize,
}

impl<'a, 'b, T> Iterator for ArbitraryChunkExact<'a, 'b, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > self.counts.len() - 1 {
            return None;
        }

        let c = self.counts[self.cursor];
        self.cursor += 1;

        if c == 0 {
            return Some(&[]);
        }

        if self.data.is_empty() || c > self.data.len() {
            return None;
        }

        let slice = mem::take(&mut self.data);
        let (l, r) = slice.split_at(c);
        self.data = r;

        Some(l)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.counts.len() - self.cursor;
        (remaining, Some(remaining))
    }
}

impl<'a, 'b, T> ArbitraryChunkExact<'a, 'b, T> {
    pub fn remainder(&'a self) -> &'a [T] {
        self.data
    }
}

pub trait ArbitraryChunks<'a, 'b, T> {
    /// `arbitrary_chunks` returns an iterator over chunks of sizes defined in `counts`.
    ///
    /// ```rust
    /// use arbitrary_chunks::ArbitraryChunks;
    ///
    /// let chunks: Vec<usize> = vec![1, 3, 1];
    /// let data: Vec<i32> = vec![0, 1, 2, 3, 4];
    ///
    /// let chunked_data: Vec<Vec<i32>> = data
    ///     .arbitrary_chunks(&chunks)
    ///     .map(|chunk| chunk.to_vec())
    ///     .collect();
    ///
    /// assert_eq!(vec![0], chunked_data[0]);
    /// assert_eq!(vec![1, 2, 3], chunked_data[1]);
    /// assert_eq!(vec![4], chunked_data[2]);
    /// ```
    fn arbitrary_chunks(&'a self, counts: &'b [usize]) -> ArbitraryChunk<'a, 'b, T>;

    /// `arbitrary_chunks_mut` returns an iterator over mutable chunks of sizes defined in `counts`.
    ///
    /// ```rust
    /// use arbitrary_chunks::ArbitraryChunks;
    ///
    /// let chunks: Vec<usize> = vec![1, 3, 1];
    /// let mut data: Vec<i32> = vec![0, 1, 2, 3, 4];
    ///
    /// data
    ///     .arbitrary_chunks_mut(&chunks)
    ///     .for_each(|chunk| {
    ///         chunk[0] = chunk[0] * 2;
    ///     });
    ///
    /// assert_eq!(vec![0, 2, 2, 3, 8], data);
    /// ```
    fn arbitrary_chunks_mut(&'a mut self, counts: &'b [usize]) -> ArbitraryChunkMut<'a, 'b, T>;

    /// `arbitrary_chunks_exact` returns chunks sized exactly as requested, or not at all.
    /// If there is not enough data to satisfy the chunk, the iterator will end. You will then be
    /// able to get the remainder of the data using `.remainder()` on the iterator.
    ///
    /// ```rust
    /// use arbitrary_chunks::ArbitraryChunks;
    ///
    /// let chunks: Vec<usize> = vec![1, 3];
    /// let data: Vec<i32> = vec![0, 1, 2];
    /// let mut iter = data.arbitrary_chunks_exact(&chunks);
    ///
    /// assert_eq!(vec![0], iter.next().unwrap());
    /// assert_eq!(None, iter.next());
    /// assert_eq!(vec![1, 2], iter.remainder());
    /// ```
    fn arbitrary_chunks_exact(&'a self, counts: &'b [usize]) -> ArbitraryChunkExact<'a, 'b, T>;

    /// `arbitrary_chunks_exact_mut` returns chunks sized exactly as requested, or not at all.
    /// If there is not enough data to satisfy the chunk, the iterator will end. You will then be
    /// able to get the remainder of the data using `.remainder()` on the iterator.
    ///
    /// ```rust
    /// use arbitrary_chunks::ArbitraryChunks;
    ///
    /// let chunks: Vec<usize> = vec![1, 3];
    /// let mut data: Vec<i32> = vec![0, 1, 2];
    /// let mut iter = data.arbitrary_chunks_exact_mut(&chunks);
    ///
    /// assert_eq!(vec![0], iter.next().unwrap());
    /// assert_eq!(None, iter.next());
    /// assert_eq!(vec![1, 2], iter.remainder());
    /// ```
    fn arbitrary_chunks_exact_mut(&'a mut self, counts: &'b [usize]) -> ArbitraryChunkExactMut<'a, 'b, T>;
}

impl<'a, 'b, T> ArbitraryChunks<'a, 'b, T> for [T] {
    fn arbitrary_chunks(&'a self, counts: &'b [usize]) -> ArbitraryChunk<'a, 'b, T> {
        ArbitraryChunk {
            data: self,
            counts,
            cursor: 0,
        }
    }

    fn arbitrary_chunks_mut(&'a mut self, counts: &'b [usize]) -> ArbitraryChunkMut<'a, 'b, T> {
        ArbitraryChunkMut {
            data: self,
            counts,
            cursor: 0,
        }
    }

    fn arbitrary_chunks_exact(&'a self, counts: &'b [usize]) -> ArbitraryChunkExact<'a, 'b, T> {
        ArbitraryChunkExact {
            data: self,
            counts,
            cursor: 0,
        }
    }

    fn arbitrary_chunks_exact_mut(&'a mut self, counts: &'b [usize]) -> ArbitraryChunkExactMut<'a, 'b, T> {
        ArbitraryChunkExactMut {
            data: self,
            counts,
            cursor: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ArbitraryChunks;

    #[test]
    fn it_stops_when_chunks_run_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let chunk_data: Vec<Vec<i32>> = data
            .arbitrary_chunks(&chunks)
            .map(|chunk| chunk.to_vec())
            .collect();

        assert_eq!(Vec::<i32>::new(), chunk_data[0]);
        assert_eq!(vec![8], chunk_data[1]);
        assert_eq!(vec![7, 6], chunk_data[2]);
        assert_eq!(vec![5, 4, 3], chunk_data[3]);
        assert_eq!(None, chunk_data.get(4));
    }

    #[test]
    fn mut_stops_when_chunks_run_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let mut data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let chunk_data: Vec<Vec<i32>> = data
            .arbitrary_chunks_mut(&chunks)
            .map(|chunk| chunk.to_vec())
            .collect();

        assert_eq!(Vec::<i32>::new(), chunk_data[0]);
        assert_eq!(vec![8], chunk_data[1]);
        assert_eq!(vec![7, 6], chunk_data[2]);
        assert_eq!(vec![5, 4, 3], chunk_data[3]);
        assert_eq!(None, chunk_data.get(4));
    }

    #[test]
    fn exact_stops_when_chunks_run_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let mut iter = data.arbitrary_chunks_exact(&chunks);

        assert_eq!(&[0i32; 0], iter.next().unwrap());
        assert_eq!(&[8i32], iter.next().unwrap());
        assert_eq!(&[7i32, 6], iter.next().unwrap());
        assert_eq!(&[5i32, 4, 3], iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(&[2i32, 1], iter.remainder());
    }

    #[test]
    fn exact_mut_stops_when_chunks_run_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let mut data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let mut iter = data.arbitrary_chunks_exact_mut(&chunks);

        assert_eq!(&mut [0i32; 0], iter.next().unwrap());
        assert_eq!(&mut [8i32], iter.next().unwrap());
        assert_eq!(&mut [7i32, 6], iter.next().unwrap());
        assert_eq!(&mut [5i32, 4, 3], iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(&mut [2i32, 1], iter.remainder());
    }

    #[test]
    fn it_accounts_for_trailing_zeros() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3, 0, 0];
        let data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let chunk_data: Vec<Vec<i32>> = data
            .arbitrary_chunks(&chunks)
            .map(|chunk| chunk.to_vec())
            .collect();

        assert_eq!(Vec::<i32>::new(), chunk_data[0]);
        assert_eq!(vec![8], chunk_data[1]);
        assert_eq!(vec![7, 6], chunk_data[2]);
        assert_eq!(vec![5, 4, 3], chunk_data[3]);
        assert_eq!(Vec::<i32>::new(), chunk_data[4]);
        assert_eq!(Vec::<i32>::new(), chunk_data[5]);
        assert_eq!(None, chunk_data.get(6));
    }

    #[test]
    fn mut_accounts_for_trailing_zeros() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3, 0, 0];
        let mut data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let chunk_data: Vec<Vec<i32>> = data
            .arbitrary_chunks_mut(&chunks)
            .map(|chunk| chunk.to_vec())
            .collect();

        assert_eq!(Vec::<i32>::new(), chunk_data[0]);
        assert_eq!(vec![8], chunk_data[1]);
        assert_eq!(vec![7, 6], chunk_data[2]);
        assert_eq!(vec![5, 4, 3], chunk_data[3]);
        assert_eq!(Vec::<i32>::new(), chunk_data[4]);
        assert_eq!(Vec::<i32>::new(), chunk_data[5]);
        assert_eq!(None, chunk_data.get(6));
    }

    #[test]
    fn exact_accounts_for_trailing_zeros() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3, 0, 0];
        let data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let mut iter = data.arbitrary_chunks_exact(&chunks);

        assert_eq!(&[0i32; 0], iter.next().unwrap());
        assert_eq!(&[8i32], iter.next().unwrap());
        assert_eq!(&[7i32, 6], iter.next().unwrap());
        assert_eq!(&[5i32, 4, 3], iter.next().unwrap());
        assert_eq!(&[0i32; 0], iter.next().unwrap());
        assert_eq!(&[0i32; 0], iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(&[2i32, 1], iter.remainder());
    }

    #[test]
    fn exact_mut_accounts_for_trailing_zeros() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3, 0, 0];
        let mut data = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let mut iter = data.arbitrary_chunks_exact_mut(&chunks);

        assert_eq!(&mut [0i32; 0], iter.next().unwrap());
        assert_eq!(&mut [8i32], iter.next().unwrap());
        assert_eq!(&mut [7i32, 6], iter.next().unwrap());
        assert_eq!(&mut [5i32, 4, 3], iter.next().unwrap());
        assert_eq!(&[0i32; 0], iter.next().unwrap());
        assert_eq!(&[0i32; 0], iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(&mut [2i32, 1], iter.remainder());
    }

    #[test]
    fn it_stops_when_data_runs_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let data = vec![8, 7, 6, 5, 4];

        let chunk_data: Vec<Vec<i32>> = data
            .arbitrary_chunks(&chunks)
            .map(|chunk| chunk.to_vec())
            .collect();

        assert_eq!(Vec::<i32>::new(), chunk_data[0]);
        assert_eq!(vec![8], chunk_data[1]);
        assert_eq!(vec![7, 6], chunk_data[2]);
        assert_eq!(vec![5, 4], chunk_data[3]);
        assert_eq!(None, chunk_data.get(4));
    }

    #[test]
    fn mut_stops_when_data_runs_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let mut data = vec![8, 7, 6, 5, 4];
        let chunk_data: Vec<Vec<i32>> = data
            .arbitrary_chunks_mut(&chunks)
            .map(|chunk| chunk.to_vec())
            .collect();

        assert_eq!(Vec::<i32>::new(), chunk_data[0]);
        assert_eq!(vec![8], chunk_data[1]);
        assert_eq!(vec![7, 6], chunk_data[2]);
        assert_eq!(vec![5, 4], chunk_data[3]);
        assert_eq!(None, chunk_data.get(4));
    }

    #[test]
    fn exact_stops_when_data_runs_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let data = vec![8, 7, 6, 5, 4];
        let mut iter = data.arbitrary_chunks_exact(&chunks);

        assert_eq!(&[0i32; 0], iter.next().unwrap());
        assert_eq!(&[8i32], iter.next().unwrap());
        assert_eq!(&[7i32, 6], iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(&[5i32, 4], iter.remainder());
    }

    #[test]
    fn exact_mut_stops_when_data_runs_out() {
        let chunks: Vec<usize> = vec![0, 1, 2, 3];
        let mut data = vec![8, 7, 6, 5, 4];
        let mut iter = data.arbitrary_chunks_exact_mut(&chunks);

        assert_eq!(&mut [0i32; 0], iter.next().unwrap());
        assert_eq!(&mut [8i32], iter.next().unwrap());
        assert_eq!(&mut [7i32, 6], iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(&mut [5i32, 4], iter.remainder());
    }
}
