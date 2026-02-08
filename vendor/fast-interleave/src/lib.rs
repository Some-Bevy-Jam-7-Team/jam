#![allow(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::{num::NonZeroUsize, ops::Range};

/// Interleave the given channels into the output buffer.
///
/// * `input` - The de-interleaved input channels.
/// * `input_range` - The range in each input channel to read from.
/// * `output` - The interleaved output buffer to write to.
///
/// # Example
///
/// ```
/// # use fast_interleave::*;
/// let input: Vec<Vec<i32>> = vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]];
/// let mut output: Vec<i32> = vec![0; 8];
///
/// interleave::<_, _, 2>(&input, 0..4, &mut output);
///
/// assert_eq!(&output, &[1, 2, 3, 4, 5, 6, 7, 8]);
///
/// // Interleave range in input:
/// interleave::<_, _, 2>(&input, 2..4, &mut output[0..4]);
///
/// assert_eq!(&output, &[5, 6, 7, 8, 5, 6, 7, 8]);
/// ```
///
/// # Panics
/// Panics when any of the following are true:
/// * `input.len() < CHANNELS`
/// * `input_range` is out-of-bounds for any input channel
/// * `output.len() < (input_range.end - input_range.start) * CHANNELS`
/// * `CHANNELS` == 0
pub fn interleave<T: Copy, Vin: AsRef<[T]>, const CHANNELS: usize>(
    input: &[Vin],
    input_range: Range<usize>,
    output: &mut [T],
) {
    assert!(CHANNELS > 0);
    assert!(input.len() >= CHANNELS);

    if input_range.is_empty() {
        return;
    }

    assert!(output.len() >= (input_range.end - input_range.start) * CHANNELS);

    for ch in input.iter() {
        let ch = ch.as_ref();
        assert!(input_range.start <= ch.len());
        assert!(input_range.end <= ch.len());
    }

    // SAFETY: All of the required safety conditions were checked above.
    unsafe {
        interleave_unchecked::<T, _, CHANNELS>(input, input_range, output);
    }
}

/// Interleave the given channels into the output buffer.
///
/// No bounds checking will occur.
///
/// * `input` - The de-interleaved input channels.
/// * `input_range` - The range in each input channel to read from.
/// * `output` - The interleaved output buffer to write to.
///
/// # Example
///
/// ```
/// # use fast_interleave::*;
/// let input: Vec<Vec<i32>> = vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]];
/// let mut output: Vec<i32> = vec![0; 8];
///
/// unsafe { interleave_unchecked::<_, _, 2>(&input, 0..4, &mut output); }
///
/// assert_eq!(&output, &[1, 2, 3, 4, 5, 6, 7, 8]);
///
/// // Interleave range in input:
/// unsafe { interleave_unchecked::<_, _, 2>(&input, 2..4, &mut output[0..4]); }
///
/// assert_eq!(&output, &[5, 6, 7, 8, 5, 6, 7, 8]);
/// ```
///
/// # Safety
/// The caller must uphold that all of the following conditions are true:
/// * `input.len() >= CHANNELS`
/// * `input_range` is within bounds for every input channel
/// * `output.len() >= (input_range.end - input_range.start) * CHANNELS`
/// * `CHANNELS` > 0
pub unsafe fn interleave_unchecked<T: Copy, Vin: AsRef<[T]>, const CHANNELS: usize>(
    input: &[Vin],
    input_range: Range<usize>,
    output: &mut [T],
) {
    if input_range.is_empty() {
        return;
    }

    unsafe {
        for i in 0..(input_range.end - input_range.start) {
            for ch_i in 0..CHANNELS {
                *output.get_unchecked_mut((i * CHANNELS) + ch_i) = *input
                    .get_unchecked(ch_i)
                    .as_ref()
                    .get_unchecked(input_range.start + i);
            }
        }
    }
}

/// De-interleave the given data into the output buffer channels.
///
/// * `input` - The interleaved input channels.
/// * `output` - The de-interleaved output channels to write to.
/// * `output_range` - The range in each output channel to write to.
///
/// # Example
///
/// ```
/// # use fast_interleave::*;
/// let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
/// let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4]];
///
/// deinterleave::<_, _, 2>(&input, &mut output, 0..4);
///
/// assert_eq!(&output[0], &[1, 3, 5, 7]);
/// assert_eq!(&output[1], &[2, 4, 6, 8]);
///
/// // Deinterleave into a range in output:
/// deinterleave::<_, _, 2>(&input[0..4], &mut output, 2..4);
///
/// assert_eq!(&output[0], &[1, 3, 1, 3]);
/// assert_eq!(&output[1], &[2, 4, 2, 4]);
/// ```
///
/// # Panics
/// Panics when any of the following are true:
/// * `output.len() < CHANNELS`
/// * `output_range` is out-of-bounds for any output channel
/// * `input.len() < (output_range.end - output_range.start) * CHANNELS`
/// * `CHANNELS` == 0
pub fn deinterleave<T: Copy, Vout: AsMut<[T]>, const CHANNELS: usize>(
    input: &[T],
    output: &mut [Vout],
    output_range: Range<usize>,
) {
    assert!(CHANNELS > 0);
    assert!(output.len() >= CHANNELS);

    if output_range.is_empty() {
        return;
    }

    assert!(input.len() >= (output_range.end - output_range.start) * CHANNELS);

    for ch in output.iter_mut() {
        let ch = ch.as_mut();
        assert!(output_range.start <= ch.len());
        assert!(output_range.end <= ch.len());
    }

    // SAFETY: All of the required safety conditions were checked above.
    unsafe {
        deinterleave_unchecked::<T, _, CHANNELS>(input, output, output_range);
    }
}

/// De-interleave the given channels into the output buffer.
///
/// No bounds checking will occur.
///
/// * `input` - The interleaved input channels.
/// * `output` - The de-interleaved output channels to write to.
/// * `output_range` - The range in each output channel to write to.
///
/// # Example
///
/// ```
/// # use fast_interleave::*;
/// let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
/// let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4]];
///
/// unsafe { deinterleave_unchecked::<_, _, 2>(&input, &mut output, 0..4); }
///
/// assert_eq!(&output[0], &[1, 3, 5, 7]);
/// assert_eq!(&output[1], &[2, 4, 6, 8]);
///
/// // Deinterleave into a range in output:
/// unsafe { deinterleave_unchecked::<_, _, 2>(&input[0..4], &mut output, 2..4); }
///
/// assert_eq!(&output[0], &[1, 3, 1, 3]);
/// assert_eq!(&output[1], &[2, 4, 2, 4]);
/// ```
///
/// # Safety
/// The caller must uphold that all of the following conditions are true:
/// * `output.len() >= CHANNELS`
/// * `output_range` is within bounds for every output channel
/// * `input.len() >= (output_range.end - output_range.start) * CHANNELS`
/// * `CHANNELS` > 0
pub unsafe fn deinterleave_unchecked<T: Copy, Vout: AsMut<[T]>, const CHANNELS: usize>(
    input: &[T],
    output: &mut [Vout],
    output_range: Range<usize>,
) {
    if output_range.is_empty() {
        return;
    }

    unsafe {
        for i in 0..(output_range.end - output_range.start) {
            for ch_i in 0..CHANNELS {
                *output
                    .get_unchecked_mut(ch_i)
                    .as_mut()
                    .get_unchecked_mut(output_range.start + i) =
                    *input.get_unchecked((i * CHANNELS) + ch_i)
            }
        }
    }
}

/// Interleave a variable number of input channels into the output buffer
/// with a variable number of channels.
///
/// * `input` - The input channels.
/// * `input_range` - The range in each channel in `input` to read from.
/// * `output` - The interleaved output buffer to write to.
/// * `num_out_channels` - The number of interleaved channels in `output`.
///
/// Note, if `num_out_channels.get() > input.len()`, then the extra samples in
/// each output frame will be left untouched.
///
/// # Example
///
/// ```
/// # use fast_interleave::*;
/// # use std::num::NonZeroUsize;
/// let input: Vec<Vec<i32>> = vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]];
/// let mut output: Vec<i32> = vec![0; 12];
///
/// interleave_variable(&input, 0..4, &mut output, NonZeroUsize::new(3).unwrap());
///
/// assert_eq!(&output, &[1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 8, 0]);
/// ```
///
/// # Panics
/// Panics when any of the following are true:
/// * `input_range` is out-of-bounds for any input channel
/// * `output.len() < (input_range.end - input_range.start) * num_out_channels.get()`
pub fn interleave_variable<T: Copy, Vin: AsRef<[T]>>(
    input: &[Vin],
    input_range: Range<usize>,
    output: &mut [T],
    num_out_channels: NonZeroUsize,
) {
    if input_range.is_empty() || input.is_empty() {
        return;
    }

    let in_frames = input_range.end - input_range.start;

    assert!(output.len() >= in_frames * num_out_channels.get());

    if num_out_channels.get() == 1 {
        // Mono, no need to interleave.
        output[..in_frames].copy_from_slice(&input[0].as_ref()[input_range.clone()]);

        return;
    }

    if num_out_channels.get() == 2 && input.len() >= 2 {
        // Provide an optimized loop for stereo since it is a common case.
        interleave::<T, _, 2>(input, input_range, output);
        return;
    }

    for (ch_i, in_ch) in (0..num_out_channels.get()).zip(input.iter()) {
        let in_ch = in_ch.as_ref();

        assert!(input_range.start < in_ch.len() && input_range.end <= in_ch.len());

        // SAFETY: We have checked that the output slice has sufficient length and that
        // the `input_range` is within bounds of the input channel slice.
        unsafe {
            for i in 0..in_frames {
                *output.get_unchecked_mut((i * num_out_channels.get()) + ch_i) =
                    *in_ch.get_unchecked(input_range.start + i);
            }
        }
    }
}

/// Interleave a variable number of input channels into a new interleaved `Vec`
/// with a variable number of channels.
///
/// The returned `Vec` with have a length of
/// `(input_range.end - input_range.start) * num_out_channels.get()`.
///
/// * `input` - The input channels.
/// * `input_range` - The range in each channel in `input` to read from.
/// * `num_out_channels` - The number of interleaved channels in output `Vec`.
///
/// Note, if `num_out_channels.get() > input.len()`, then the extra samples in
/// each output frame will be set to the default value.
///
/// # Example
///
/// ```
/// # use fast_interleave::*;
/// # use std::num::NonZeroUsize;
/// let input: Vec<Vec<i32>> = vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]];
/// let mut output: Vec<i32> = vec![0; 12];
///
/// let output = interleave_to_vec_variable(&input, 0..4, NonZeroUsize::new(3).unwrap());
///
/// assert_eq!(&output, &[1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 8, 0]);
/// ```
///
/// # Panics
/// Panics if `input_range` is out-of-bounds for any input channel.
#[cfg(feature = "alloc")]
pub fn interleave_to_vec_variable<T: Copy + Default, Vin: AsRef<[T]>>(
    input: &[Vin],
    input_range: Range<usize>,
    num_out_channels: NonZeroUsize,
) -> Vec<T> {
    let out_len = (input_range.end - input_range.start) * num_out_channels.get();

    let mut out: Vec<T> = Vec::new();
    out.reserve_exact(out_len);

    if num_out_channels.get() > input.len() {
        out.resize(out_len, T::default());
    } else {
        // Safety:
        // * The vec has been initialized with a capacity of `out_len` above.
        // * `interleave_variable` is gauranteed to completely intialize the output
        // slice up to `(input_range.end - input_range.start) * num_out_channels.get()`
        // when `num_out_channels.get() <= input.len()`.
        unsafe {
            out.set_len(out_len);
        }
    }

    interleave_variable(input, input_range, out.as_mut_slice(), num_out_channels);

    out
}

/// Deinterleave the given data with a variable number of channels into
/// the output buffers with a variable number of channels.
///
/// * `input` - The interleaved input data.
/// * `num_in_channels` - The number of interleaved channels in `input`.
/// * `output` - The de-interleaved output buffer to write to.
/// * `out_range` - The range in each channel in `output` to write to.
///
/// Note, if `output.len() > num_in_channels.get()`, then the extra output channels
/// will be left untouched.
///
/// # Example
///
/// ```
/// # use fast_interleave::*;
/// # use std::num::NonZeroUsize;
/// let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
/// let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4], vec![0; 4]];
///
/// deinterleave_variable(&input, NonZeroUsize::new(2).unwrap(), &mut output, 0..4);
///
/// assert_eq!(&output[0], &[1, 3, 5, 7]);
/// assert_eq!(&output[1], &[2, 4, 6, 8]);
/// assert_eq!(&output[2], &[0, 0, 0, 0]);
/// ```
///
/// # Panics
/// Panics when any of the following are true:
/// * `output_range` is out-of-bounds for any output channel
/// * `input.len() < (output_range.end - output_range.start) * num_in_channels.get()`
pub fn deinterleave_variable<T: Copy, Vout: AsMut<[T]>>(
    input: &[T],
    num_in_channels: NonZeroUsize,
    output: &mut [Vout],
    output_range: Range<usize>,
) {
    if output_range.is_empty() || output.is_empty() {
        return;
    }

    let out_frames = output_range.end - output_range.start;

    if num_in_channels.get() == 1 {
        // Mono, no need to deinterleave.
        output[0].as_mut()[output_range.clone()].copy_from_slice(&input[..out_frames]);

        return;
    }

    if num_in_channels.get() == 2 && output.len() >= 2 {
        // Provide an optimized loop for stereo since it is a common case.
        deinterleave::<T, _, 2>(input, output, output_range);
        return;
    }

    assert!(input.len() >= (output_range.end - output_range.start) * num_in_channels.get());

    for (ch_i, out_ch) in (0..num_in_channels.get()).zip(output.iter_mut()) {
        let out_ch = out_ch.as_mut();

        assert!(output_range.start < out_ch.len() && output_range.end <= out_ch.len());

        // SAFETY: We have checked that the input slice has sufficient length and that
        // the `output_range` is within bounds of the output channel slice.
        unsafe {
            for i in 0..(output_range.end - output_range.start) {
                *out_ch.get_unchecked_mut(output_range.start + i) =
                    *input.get_unchecked((i * num_in_channels.get()) + ch_i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{vec, vec::Vec};

    use super::*;

    #[test]
    fn test_interleave() {
        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4]];
        let mut output: Vec<i32> = vec![0; 4];
        interleave::<_, _, 1>(&input, 1..4, &mut output);
        assert_eq!(&output, &[2, 3, 4, 0]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let mut output: Vec<i32> = vec![0; 8];
        interleave::<_, _, 2>(&input, 1..4, &mut output);
        assert_eq!(&output, &[2, 6, 3, 7, 4, 8, 0, 0]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];
        let mut output: Vec<i32> = vec![0; 12];
        interleave::<_, _, 3>(&input, 1..4, &mut output);
        assert_eq!(&output, &[2, 6, 10, 3, 7, 11, 4, 8, 12, 0, 0, 0]);
    }

    #[test]
    fn test_deinterleave() {
        let input: Vec<i32> = vec![1, 2, 3, 4];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4]];
        deinterleave::<_, _, 1>(&input, &mut output, 1..4);
        assert_eq!(&output[0], &[0, 1, 2, 3]);

        let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4]];
        deinterleave::<_, _, 2>(&input, &mut output, 1..4);
        assert_eq!(&output[0], &[0, 1, 3, 5]);
        assert_eq!(&output[1], &[0, 2, 4, 6]);

        let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4], vec![0; 4]];
        deinterleave::<_, _, 3>(&input, &mut output, 1..4);
        assert_eq!(&output[0], &[0, 1, 4, 7]);
        assert_eq!(&output[1], &[0, 2, 5, 8]);
        assert_eq!(&output[2], &[0, 3, 6, 9]);
    }

    #[test]
    fn test_interleave_variable() {
        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4]];
        let mut output: Vec<i32> = vec![0; 4];
        interleave_variable(&input, 1..4, &mut output, NonZeroUsize::new(1).unwrap());
        assert_eq!(&output, &[2, 3, 4, 0]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let mut output: Vec<i32> = vec![0; 8];
        interleave_variable(&input, 0..4, &mut output, NonZeroUsize::new(1).unwrap());
        assert_eq!(&output, &[1, 2, 3, 4, 0, 0, 0, 0]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4]];
        let mut output: Vec<i32> = vec![0; 8];
        interleave_variable(&input, 0..4, &mut output, NonZeroUsize::new(2).unwrap());
        assert_eq!(&output, &[1, 0, 2, 0, 3, 0, 4, 0]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let mut output: Vec<i32> = vec![0; 8];
        interleave_variable(&input, 0..4, &mut output, NonZeroUsize::new(2).unwrap());
        assert_eq!(&output, &[1, 5, 2, 6, 3, 7, 4, 8]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];
        let mut output: Vec<i32> = vec![0; 8];
        interleave_variable(&input, 0..4, &mut output, NonZeroUsize::new(2).unwrap());
        assert_eq!(&output, &[1, 5, 2, 6, 3, 7, 4, 8]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];
        let mut output: Vec<i32> = vec![0; 12];
        interleave_variable(&input, 0..4, &mut output, NonZeroUsize::new(3).unwrap());
        assert_eq!(&output, &[1, 5, 9, 2, 6, 10, 3, 7, 11, 4, 8, 12]);
    }

    #[test]
    fn test_interleave_to_vec_variable() {
        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4]];
        let output = interleave_to_vec_variable(&input, 1..4, NonZeroUsize::new(1).unwrap());
        assert_eq!(&output, &[2, 3, 4]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let output = interleave_to_vec_variable(&input, 0..4, NonZeroUsize::new(1).unwrap());
        assert_eq!(&output, &[1, 2, 3, 4]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4]];
        let output = interleave_to_vec_variable(&input, 0..4, NonZeroUsize::new(2).unwrap());
        assert_eq!(&output, &[1, 0, 2, 0, 3, 0, 4, 0]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let output = interleave_to_vec_variable(&input, 0..4, NonZeroUsize::new(2).unwrap());
        assert_eq!(&output, &[1, 5, 2, 6, 3, 7, 4, 8]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];
        let output = interleave_to_vec_variable(&input, 0..4, NonZeroUsize::new(2).unwrap());
        assert_eq!(&output, &[1, 5, 2, 6, 3, 7, 4, 8]);

        let input: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];
        let output = interleave_to_vec_variable(&input, 0..4, NonZeroUsize::new(3).unwrap());
        assert_eq!(&output, &[1, 5, 9, 2, 6, 10, 3, 7, 11, 4, 8, 12]);
    }

    #[test]
    fn test_deinterleave_variable() {
        let input: Vec<i32> = vec![1, 2, 3, 4];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4]];
        deinterleave_variable(&input, NonZeroUsize::new(1).unwrap(), &mut output, 1..4);
        assert_eq!(&output[0], &[0, 1, 2, 3]);

        let input: Vec<i32> = vec![1, 2, 3, 4];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4]];
        deinterleave_variable(&input, NonZeroUsize::new(1).unwrap(), &mut output, 0..4);
        assert_eq!(&output[0], &[1, 2, 3, 4]);
        assert_eq!(&output[1], &[0, 0, 0, 0]);

        let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4]];
        deinterleave_variable(&input, NonZeroUsize::new(2).unwrap(), &mut output, 0..4);
        assert_eq!(&output[0], &[1, 3, 5, 7]);

        let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4]];
        deinterleave_variable(&input, NonZeroUsize::new(2).unwrap(), &mut output, 0..4);
        assert_eq!(&output[0], &[1, 3, 5, 7]);
        assert_eq!(&output[1], &[2, 4, 6, 8]);

        let input: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let mut output: Vec<Vec<i32>> = vec![vec![0; 4], vec![0; 4], vec![0; 4]];
        deinterleave_variable(&input, NonZeroUsize::new(3).unwrap(), &mut output, 0..4);
        assert_eq!(&output[0], &[1, 4, 7, 10]);
        assert_eq!(&output[1], &[2, 5, 8, 11]);
        assert_eq!(&output[2], &[3, 6, 9, 12]);
    }
}
