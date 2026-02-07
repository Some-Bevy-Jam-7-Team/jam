//! # block-pseudorand
//!
//! This crate allows multi-threaded creation of pseudorandom `Vec<T>`'s of arbitrary length. It does this
//! by generating arbitrary byte vectors randomly and transmuting to the provided type.
//!
//! <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
//! <strong>Warning:</strong> This is wildly unsafe for some types as it does not uphold any invariants your type might expect.
//! Only use this crate if your type can be safely generated from completely arbitrary bytes.
//! Generally, this means your type should consist of nothing but primitive numbers such as u32, i64, or f32.
//! </p>
//!
//! ## Notes
//!
//!  - Generated data is not guaranteed to be cryptographically secure
//!  - Output can be deterministic if you provide a seed
//!  - This is very unsafe for certain types, see the warning above
//!
//! ## Use cases
//!
//! If you need a lot of random numbers quickly for something non-production critical like unit tests, this may be
//! a good candidate. Otherwise, if you are planning to use this at runtime, or with types that are non-numeric
//! or otherwise cannot be created from arbitrary bytes, I would recommend you to choose another, safer crate.
//!
//! ## Usage
//!
//! If you are certain the above warnings do not apply to the type you are generating, you can use this library like so:
//!
//! ### Without Seed
//!
//! ```rust
//! use block_pseudorand::block_rand;
//!
//! let random_data: Vec<u64> = block_rand(128);
//!
//! assert_eq!(random_data.len(), 128);
//! ```
//!
//! ### With Seed
//!
//! ```rust
//! use block_pseudorand::block_rand_with_seed;
//!
//! // Populate this seed as you wish
//! let seed = [0u8; 32];
//! let random_data: Vec<u64> = block_rand_with_seed(128, &seed);
//!
//! assert_eq!(random_data.len(), 128);
//! ```

use std::mem::size_of;
use chiapos_chacha8::ChaCha8;
use nanorand::{Rng, WyRand};

#[inline]
const fn cdiv(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

/// Randomly populates a `Vec` of data based upon a provided seed.
///
/// Usage:
///
/// ```rust
/// use block_pseudorand::block_rand_with_seed;
///
/// let seed = [0u8; 32];
/// let random_data: Vec<u32> = block_rand_with_seed(128, &seed);
///
/// assert_eq!(random_data.len(), 128);
/// ```
pub fn block_rand_with_seed<T: Copy>(count: usize, seed: &[u8; 32]) -> Vec<T> {
    let expected_len_bytes = count * size_of::<T>();
    let gen_len = cdiv(expected_len_bytes, 64) * 64;

    let mut bytes = Vec::with_capacity(gen_len);
    unsafe {
        bytes.set_len(gen_len);
    };

    let chacha8 = ChaCha8::new_from_256bit_key(seed);
    chacha8.get_keystream(0, &mut bytes);

    bytes.truncate(expected_len_bytes);

    let mut out = std::mem::ManuallyDrop::new(bytes);

    unsafe {
        Vec::from_raw_parts(
            out.as_mut_ptr() as *mut T,
            count,
            count
        )
    }
}

/// Randomly populates a `Vec` of data based upon a randomly chosen seed.
///
/// Usage:
///
/// ```rust
/// use block_pseudorand::block_rand;
///
/// let random_data: Vec<u32> = block_rand(128);
///
/// assert_eq!(random_data.len(), 128);
/// ```
pub fn block_rand<T: Copy>(count: usize) -> Vec<T> {
    let mut rng = WyRand::new();
    let mut key = [0u8; 32];
    for v in key.iter_mut() {
        *v = rng.generate();
    }

    block_rand_with_seed(count, &key)
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn test_count<T: Copy>(count: usize) {
        let rand_data: Vec<T> = block_rand(count);
        assert_eq!(rand_data.len(), count);
    }

    fn test_medley<T: Copy>() {
        test_count::<T>(0);
        test_count::<T>(1);
        test_count::<T>(2);
        test_count::<T>(3);
        test_count::<T>(10_000);
        test_count::<T>(1_000_000);
        test_count::<T>(10_000_000);
    }

    #[test]
    fn u8_works() {
        test_medley::<u8>();
    }

    #[test]
    fn u16_works() {
        test_medley::<u16>();
    }

    #[test]
    fn u32_works() {
        test_medley::<u32>();
    }

    #[test]
    fn u64_works() {
        test_medley::<u64>();
    }

    #[test]
    fn u128_works() {
        test_medley::<u128>();
    }

    #[test]
    fn usize_works() {
        test_medley::<usize>();
    }

    #[test]
    fn i8_works() {
        test_medley::<i8>();
    }

    #[test]
    fn i16_works() {
        test_medley::<i16>();
    }

    #[test]
    fn i32_works() {
        test_medley::<i32>();
    }

    #[test]
    fn i64_works() {
        test_medley::<i64>();
    }

    #[test]
    fn i128_works() {
        test_medley::<i128>();
    }

    #[test]
    fn isize_works() {
        test_medley::<isize>();
    }

    #[test]
    fn f32_works() {
        test_medley::<f32>();
    }

    #[test]
    fn f64_works() {
        test_medley::<f64>();
    }
}
