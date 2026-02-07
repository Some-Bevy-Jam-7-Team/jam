#[cfg(not(feature = "std"))]
use num_traits::Float;

pub mod butterworth;
pub mod single_pole_iir;
pub mod smoothing_filter;
pub mod svf;

/// Convert bandwidth in Hz to "q factor"
pub fn bandwidth_hz_to_q(bandwidth_hz: f32, cutoff_hz: f32) -> f32 {
    cutoff_hz / bandwidth_hz
}

/// Convert "q factor" in bandwidth in Hz
pub fn q_to_bandwidth_hz(q: f32, cutoff_hz: f32) -> f32 {
    cutoff_hz / q
}

/// Convert bandwidth in octaves to "q factor"
pub fn bandwidth_octaves_to_q(bandwidth_octaves: f32) -> f32 {
    let two_pow_bw = 2.0f32.powf(bandwidth_octaves);
    two_pow_bw.sqrt() / (two_pow_bw - 1.0)
}
