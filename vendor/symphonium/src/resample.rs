use std::{
    collections::HashMap,
    fmt::Debug,
    num::{NonZeroU32, NonZeroUsize},
};

use crate::MAX_CHANNELS;

pub use fixed_resample;

use fixed_resample::FixedResampler;
pub use fixed_resample::ResampleQuality;

/// The parameters to get a custom resampler
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResamplerParams {
    pub num_channels: NonZeroUsize,
    pub source_sample_rate: NonZeroU32,
    pub target_sample_rate: NonZeroU32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ResamplerKey {
    pcm_sr: NonZeroU32,
    target_sr: NonZeroU32,
    channels: u16,
    quality: u16,
}

pub(crate) fn get_resampler<'a>(
    resamplers: &'a mut HashMap<ResamplerKey, FixedResampler<f32, MAX_CHANNELS>>,
    resample_quality: ResampleQuality,
    pcm_sr: NonZeroU32,
    target_sr: NonZeroU32,
    num_channels: NonZeroUsize,
) -> &'a mut FixedResampler<f32, MAX_CHANNELS> {
    let key = ResamplerKey {
        pcm_sr,
        target_sr,
        channels: num_channels.get() as u16,
        quality: match resample_quality {
            ResampleQuality::Low => 0,
            _ => 1,
        },
    };

    resamplers.entry(key).or_insert_with(|| {
        FixedResampler::new(
            num_channels,
            pcm_sr.get(),
            target_sr.get(),
            resample_quality,
            false,
        )
    })
}
