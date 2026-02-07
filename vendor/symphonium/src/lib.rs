//! An unofficial easy-to-use wrapper around [Symphonia](https://github.com/pdeljanov/Symphonia)
//! for loading audio files. It also handles resampling at load-time.
//!
//! The resulting `DecodedAudio` resources are stored in their native sample format whenever
//! possible to save on memory, and have convenience methods to fill a buffer with `f32` samples
//! from any arbitrary position in the resource in realtime during playback. Alternatively you
//! can use the `DecodedAudioF32` resource if you only need samples in the `f32` format.
//!
//! ## Example
//!
//! ```ignore
//! /// A struct used to load audio files.
//! let mut loader = SymphoniumLoader::new();
//!
//! let target_sample_rate = NonZeroU32::new(44100).unwrap();
//!
//! /// Load an audio file.
//! let audio_data = loader
//!     .load(
//!         // The path to the audio file.
//!         file_path,    
//!         // The target sample rate. If this differs from the
//!         // file's sample rate, then it will be resampled.
//!         // If you wish to never resample, set this to `None`.
//!         Some(target_sample_rate),
//!         // The quality of the resampling algorithm. Normal
//!         // is recommended for most applications.
//!         ResampleQuality::Normal,
//!         // The maximum size a file can be in bytes before an
//!         // error is returned. This is to protect against
//!         // out of memory errors when loading really long
//!         // audio files. Set to `None` to use the default of
//!         // 1 GB.
//!         None
//!     )
//!     .unwrap();
//!
//! /// Fill a stereo buffer with samples starting at frame 100.
//! let mut buf_l = vec![0.0f32; 512];
//! let mut buf_r = vec![0.0f32; 512];
//! audio_data.fill_stereo(100, &mut buf_l, &mut buf_r);
//!
//! /// Alternatively, if you don't need to save memory, you can
//! /// load directly to an `f32` format.
//! let audio_data_f32 = loader
//!     .load_f32(
//!         file_path,
//!         Some(target_sample_rate),
//!         ResampleQuality::Normal,
//!         None
//!     )
//!     .unwrap();
//!
//! /// Print info about the data (`data` is a `Vec<Vec<f32>>`).
//! println!("num channels: {}" audio_data_f32.data.len());
//! println!("num frames: {}" audio_data_f32.data[0].len());
//! ```
//! ## Features
//!
//! By default, only `wav` and `ogg` support is enabled. If you need more formats, enable them
//! as features in your `Cargo.toml` file like this:
//!
//! `symphonium = { version = "0.7.1", features = ["mp3", "flac"] }`
//!
//! Available codecs:
//!
//! * `aac`
//! * `adpcm`
//! * `alac`
//! * `flac`
//! * `mp1`
//! * `mp2`
//! * `mp3`
//! * `pcm`
//! * `vorbis`
//!
//! Available container formats:
//!
//! * `caf`
//! * `isomp4`
//! * `mkv`
//! * `ogg`
//! * `aiff`
//! * `wav`
//!
//! Alternatively you can enable the `all` feature if you want everything, or the `open-standards`
//! feature if you want all of the royalty-free open-source standards.

use std::fs::File;
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::Path;

#[cfg(feature = "resampler")]
use std::collections::HashMap;

use symphonia::core::codecs::CodecRegistry;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, Probe, ProbeResult};

#[cfg(all(feature = "log", not(feature = "tracing")))]
use log::warn;
#[cfg(feature = "tracing")]
use tracing::warn;

// Re-export symphonia
pub use symphonia;

pub mod error;

#[cfg(feature = "resampler")]
pub mod resample;
#[cfg(feature = "resampler")]
pub use resample::ResampleQuality;
#[cfg(feature = "resampler")]
use resample::{ResamplerKey, ResamplerParams};

mod decode;
mod resource;

pub use resource::*;

use error::LoadError;

/// The default maximum size of an audio file in bytes.
pub static DEFAULT_MAX_BYTES: usize = 1_000_000_000;

#[cfg(feature = "resampler")]
const MAX_CHANNELS: usize = 16;

/// Used to load audio files into RAM. This stores samples in
/// their native sample format when possible to save memory.
pub struct SymphoniumLoader {
    // Re-use resamplers to improve performance.
    #[cfg(feature = "resampler")]
    resamplers: HashMap<ResamplerKey, fixed_resample::FixedResampler<f32, MAX_CHANNELS>>,

    codec_registry: &'static CodecRegistry,
    probe: &'static Probe,
}

impl SymphoniumLoader {
    /// Construct a new audio file loader.
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "resampler")]
            resamplers: HashMap::new(),
            codec_registry: symphonia::default::get_codecs(),
            probe: symphonia::default::get_probe(),
        }
    }

    pub fn with_codec_registry_and_probe(
        codec_registry: &'static CodecRegistry,
        probe: &'static Probe,
    ) -> Self {
        Self {
            #[cfg(feature = "resampler")]
            resamplers: HashMap::new(),
            codec_registry,
            probe,
        }
    }

    /// Load an audio file from the given path.
    ///
    /// * `path` - The path to the audio file stored on disk.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. (No resampling will occur if the audio file's sample rate is already
    /// the target sample rate). If this is `None`, then the file will not be resampled
    /// and it will stay its original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    pub fn load<P: AsRef<Path>>(
        &mut self,
        path: P,
        #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
        #[cfg(feature = "resampler")] resample_quality: ResampleQuality,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudio, LoadError> {
        let probed = self.probe_from_file(path)?;
        self.decode_probed(
            probed,
            #[cfg(feature = "resampler")]
            target_sample_rate,
            #[cfg(feature = "resampler")]
            resample_quality,
            max_bytes,
        )
    }

    /// Load an audio source from RAM.
    ///
    /// * `source` - The audio source which implements the [`MediaSource`] trait.
    /// * `hint` - An optional hint to help the format registry guess what format reader is
    /// appropriate.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. (No resampling will occur if the audio file's sample rate is already
    /// the target sample rate). If this is `None`, then the file will not be resampled
    /// and it will stay its original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    pub fn load_from_source(
        &mut self,
        source: Box<dyn MediaSource>,
        hint: Option<Hint>,
        #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
        #[cfg(feature = "resampler")] resample_quality: ResampleQuality,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudio, LoadError> {
        let probed = self.probe_from_source(source, hint)?;
        self.decode_probed(
            probed,
            #[cfg(feature = "resampler")]
            target_sample_rate,
            #[cfg(feature = "resampler")]
            resample_quality,
            max_bytes,
        )
    }

    /// Load an audio file from the given path using a custom resampler.
    ///
    /// * `path` - The path to the audio file stored on disk.
    /// * `target_sample_rate` - The target sample rate. (No resampling will occur if the audio
    /// file's sample rate is already the target sample rate).
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    /// * `get_resampler` - Get the custom sampler with the desired parameters.
    #[cfg(feature = "resampler")]
    pub fn load_with_resampler<'a, P: AsRef<Path>>(
        &mut self,
        path: P,
        target_sample_rate: NonZeroU32,
        max_bytes: Option<usize>,
        get_resampler: impl FnOnce(
            ResamplerParams,
        ) -> &'a mut fixed_resample::FixedResampler<f32, MAX_CHANNELS>,
    ) -> Result<DecodedAudio, LoadError> {
        let probed = self.probe_from_file(path)?;
        self.decode_probed_with_resampler(probed, target_sample_rate, max_bytes, get_resampler)
    }

    /// Load an audio source from RAM using a custom resampler.
    ///
    /// * `source` - The audio source which implements the [`MediaSource`] trait.
    /// * `hint` - An optional hint to help the format registry guess what format reader is
    /// appropriate.
    /// * `target_sample_rate` - The target sample rate. (No resampling will occur if the audio
    /// file's sample rate is already the target sample rate).
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    /// * `get_resampler` - Get the custom sampler with the desired parameters.
    #[cfg(feature = "resampler")]
    pub fn load_from_source_with_resampler<'a>(
        &mut self,
        source: Box<dyn MediaSource>,
        hint: Option<Hint>,
        target_sample_rate: NonZeroU32,
        max_bytes: Option<usize>,
        get_resampler: impl FnOnce(
            ResamplerParams,
        ) -> &'a mut fixed_resample::FixedResampler<f32, MAX_CHANNELS>,
    ) -> Result<DecodedAudio, LoadError> {
        let probed = self.probe_from_source(source, hint)?;
        self.decode_probed_with_resampler(probed, target_sample_rate, max_bytes, get_resampler)
    }

    /// Load an audio file from the given path and convert to an f32 sample format.
    ///
    /// * `path` - The path to the audio file stored on disk.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. (No resampling will occur if the audio file's sample rate is already
    /// the target sample rate). If this is `None`, then the file will not be resampled
    /// and it will stay its original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    pub fn load_f32<P: AsRef<Path>>(
        &mut self,
        path: P,
        #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
        #[cfg(feature = "resampler")] resample_quality: ResampleQuality,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudioF32, LoadError> {
        let probed = self.probe_from_file(path)?;
        self.decode_probed_f32(
            probed,
            #[cfg(feature = "resampler")]
            target_sample_rate,
            #[cfg(feature = "resampler")]
            resample_quality,
            max_bytes,
        )
    }

    /// Load an audio source from RAM and convert to an f32 sample format.
    ///
    /// * `source` - The audio source which implements the [`MediaSource`] trait.
    /// * `hint` - An optional hint to help the format registry guess what format reader is
    /// appropriate.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. (No resampling will occur if the audio file's sample rate is already
    /// the target sample rate). If this is `None`, then the file will not be resampled
    /// and it will stay its original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    pub fn load_f32_from_source(
        &mut self,
        source: Box<dyn MediaSource>,
        hint: Option<Hint>,
        #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
        #[cfg(feature = "resampler")] resample_quality: ResampleQuality,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudioF32, LoadError> {
        let probed = self.probe_from_source(source, hint)?;
        self.decode_probed_f32(
            probed,
            #[cfg(feature = "resampler")]
            target_sample_rate,
            #[cfg(feature = "resampler")]
            resample_quality,
            max_bytes,
        )
    }

    /// Load an audio file from the given path using a custom resampler and convert to an f32
    /// sample format.
    ///
    /// * `path` - The path to the audio file stored on disk.
    /// * `target_sample_rate` - The target sample rate. (No resampling will occur if the audio
    /// file's sample rate is already the target sample rate).
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    /// * `get_resampler` - Get the custom sampler with the desired parameters.
    #[cfg(feature = "resampler")]
    pub fn load_f32_with_resampler<'a, P: AsRef<Path>>(
        &mut self,
        path: P,
        target_sample_rate: NonZeroU32,
        max_bytes: Option<usize>,
        get_resampler: impl FnOnce(
            ResamplerParams,
        ) -> &'a mut fixed_resample::FixedResampler<f32, MAX_CHANNELS>,
    ) -> Result<DecodedAudioF32, LoadError> {
        let probed = self.probe_from_file(path)?;
        self.decode_probed_f32_with_resampler(probed, target_sample_rate, max_bytes, get_resampler)
    }

    /// Load an audio source from RAM using a custom resampler and convert to an f32 sample
    /// format.
    ///
    /// * `source` - The audio source which implements the [`MediaSource`] trait.
    /// * `hint` - An optional hint to help the format registry guess what format reader is
    /// appropriate.
    /// * `target_sample_rate` - The target sample rate. (No resampling will occur if the audio
    /// file's sample rate is already the target sample rate).
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    /// * `get_resampler` - Get the custom sampler with the desired parameters.
    #[cfg(feature = "resampler")]
    pub fn load_f32_from_source_with_resampler<'a>(
        &mut self,
        source: Box<dyn MediaSource>,
        hint: Option<Hint>,
        target_sample_rate: NonZeroU32,
        max_bytes: Option<usize>,
        get_resampler: impl FnOnce(
            ResamplerParams,
        ) -> &'a mut fixed_resample::FixedResampler<f32, MAX_CHANNELS>,
    ) -> Result<DecodedAudioF32, LoadError> {
        let probed = self.probe_from_source(source, hint)?;
        self.decode_probed_f32_with_resampler(probed, target_sample_rate, max_bytes, get_resampler)
    }

    /// Load an audio file from the given path and convert to an f32 sample format. The sample will
    /// be stretched (pitch shifted) by the given amount.
    ///
    /// * `source` - The audio source which implements the [`MediaSource`] trait.
    /// * `stretch` - The amount of stretching (`new_length / old_length`). A value of `1.0` is no
    /// change, a value less than `1.0` will increase the pitch & decrease the length, and a value
    /// greater than `1.0` will decrease the pitch & increase the length. If a `target_sample_rate`
    /// is given, then the final amount will automatically be adjusted to account for that.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. If this is `None`, then the file will not be resampled and it will stay its
    /// original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    #[cfg(feature = "stretch-sinc-resampler")]
    pub fn load_stretched<P: AsRef<Path>>(
        &mut self,
        path: P,
        stretch: f64,
        target_sample_rate: Option<NonZeroU32>,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudioF32, LoadError> {
        let probed = self.probe_from_file(path)?;
        self.decode_probed_stretched(probed, stretch, target_sample_rate, max_bytes)
    }

    /// Load an audio source from RAM and convert to an f32 sample format. The sample will be
    /// stretched (pitch shifted) by the given amount.
    ///
    /// * `source` - The audio source which implements the [`MediaSource`] trait.
    /// * `hint` - An optional hint to help the format registry guess what format reader is
    /// appropriate.
    /// * `stretch` - The amount of stretching (`new_length / old_length`). A value of `1.0` is no
    /// change, a value less than `1.0` will increase the pitch & decrease the length, and a value
    /// greater than `1.0` will decrease the pitch & increase the length. If a `target_sample_rate`
    /// is given, then the final amount will automatically be adjusted to account for that.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. If this is `None`, then the file will not be resampled and it will stay its
    /// original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    #[cfg(feature = "stretch-sinc-resampler")]
    pub fn load_from_source_stretched(
        &mut self,
        source: Box<dyn MediaSource>,
        hint: Option<Hint>,
        stretch: f64,
        target_sample_rate: Option<NonZeroU32>,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudioF32, LoadError> {
        let probed = self.probe_from_source(source, hint)?;
        self.decode_probed_stretched(probed, stretch, target_sample_rate, max_bytes)
    }

    /// Load an audio file from the given path and probe its metadata without decoding it.
    ///
    /// This can be useful if you wish to read metadata about the audio file or override
    /// its sample rate before decoding.
    pub fn probe_from_file<P: AsRef<Path>>(&self, path: P) -> Result<ProbedAudioSource, LoadError> {
        let path: &Path = path.as_ref();

        // Try to open the file.
        let file = File::open(path)?;

        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();

        // Provide the file extension as a hint.
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        self.probe_from_source(Box::new(file), Some(hint))
    }

    /// Load an audio source from RAM and probe its metadata without decoding it.
    ///
    /// This can be useful if you wish to read metadata about the audio file or override
    /// its sample rate before decoding.
    pub fn probe_from_source(
        &self,
        source: Box<dyn MediaSource>,
        hint: Option<Hint>,
    ) -> Result<ProbedAudioSource, LoadError> {
        // Create the media source stream.
        let mss = MediaSourceStream::new(source, Default::default());

        // Use the default options for format reader, metadata reader, and decoder.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();

        let hint = hint.unwrap_or_default();

        // Probe the media source stream for metadata and get the format reader.
        let probed = self
            .probe
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| LoadError::UnkownFormat(e))?;

        // Get the default track in the audio stream.
        let track = probed
            .format
            .default_track()
            .ok_or_else(|| LoadError::NoTrackFound)?;

        let sample_rate = track.codec_params.sample_rate.and_then(|sr| {
            let sr = NonZeroU32::new(sr);
            #[cfg(any(feature = "tracing", feature = "log"))]
            {
                if sr.is_none() {
                    warn!("Audio source returned a sample rate of 0");
                }
            }
            sr
        });

        let num_channels = track
            .codec_params
            .channels
            .ok_or_else(|| LoadError::NoChannelsFound)?
            .count();

        if num_channels == 0 {
            return Err(LoadError::NoChannelsFound);
        }

        Ok(ProbedAudioSource {
            probed,
            sample_rate,
            num_channels: NonZeroUsize::new(num_channels).unwrap(),
        })
    }

    /// Decode the probed audio source.
    ///
    /// * `probed` - The probed audio source.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. (No resampling will occur if the audio file's sample rate is already
    /// the target sample rate). If this is `None`, then the file will not be resampled
    /// and it will stay its original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    pub fn decode_probed(
        &mut self,
        probed: ProbedAudioSource,
        #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
        #[cfg(feature = "resampler")] resample_quality: ResampleQuality,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudio, LoadError> {
        decode(
            probed,
            self.codec_registry,
            max_bytes,
            #[cfg(feature = "resampler")]
            target_sample_rate,
            #[cfg(feature = "resampler")]
            |params| {
                self::resample::get_resampler(
                    &mut self.resamplers,
                    resample_quality,
                    params.source_sample_rate,
                    params.target_sample_rate,
                    params.num_channels,
                )
            },
        )
    }

    /// Decode the probed audio source using a custom resampler.
    ///
    /// * `probed` - The probed audio source.
    /// * `target_sample_rate` - The target sample rate. (No resampling will occur if the audio
    /// file's sample rate is already the target sample rate).
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    /// * `get_resampler` - Get the custom sampler with the desired parameters.
    #[cfg(feature = "resampler")]
    pub fn decode_probed_with_resampler<'a>(
        &mut self,
        probed: ProbedAudioSource,
        target_sample_rate: NonZeroU32,
        max_bytes: Option<usize>,
        get_resampler: impl FnOnce(
            ResamplerParams,
        ) -> &'a mut fixed_resample::FixedResampler<f32, MAX_CHANNELS>,
    ) -> Result<DecodedAudio, LoadError> {
        decode(
            probed,
            self.codec_registry,
            max_bytes,
            Some(target_sample_rate),
            get_resampler,
        )
    }

    /// Decode the probed audio source and convert to an f32 sample format.
    ///
    /// * `probed` - The probed audio source.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. (No resampling will occur if the audio file's sample rate is already
    /// the target sample rate). If this is `None`, then the file will not be resampled
    /// and it will stay its original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    pub fn decode_probed_f32(
        &mut self,
        probed: ProbedAudioSource,
        #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
        #[cfg(feature = "resampler")] resample_quality: ResampleQuality,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudioF32, LoadError> {
        decode_f32(
            probed,
            self.codec_registry,
            max_bytes,
            #[cfg(feature = "resampler")]
            target_sample_rate,
            #[cfg(feature = "resampler")]
            |params| {
                self::resample::get_resampler(
                    &mut self.resamplers,
                    resample_quality,
                    params.source_sample_rate,
                    params.target_sample_rate,
                    params.num_channels,
                )
            },
        )
    }

    /// Decode the probed audio source and convert it to an f32 sample format, and resample
    /// with a custom resampler.
    ///
    /// * `probed` - The probed audio source.
    /// * `target_sample_rate` - The target sample rate. (No resampling will occur if the audio
    /// file's sample rate is already the target sample rate).
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    /// * `get_resampler` - Get the custom sampler with the desired parameters.
    #[cfg(feature = "resampler")]
    pub fn decode_probed_f32_with_resampler<'a>(
        &mut self,
        probed: ProbedAudioSource,
        target_sample_rate: NonZeroU32,
        max_bytes: Option<usize>,
        get_resampler: impl FnOnce(
            ResamplerParams,
        ) -> &'a mut fixed_resample::FixedResampler<f32, MAX_CHANNELS>,
    ) -> Result<DecodedAudioF32, LoadError> {
        decode_f32(
            probed,
            self.codec_registry,
            max_bytes,
            Some(target_sample_rate),
            get_resampler,
        )
    }

    /// Decode the probed audio source and convert to an f32 sample format. The sample will
    /// be stretched (pitch shifted) by the given amount.
    ///
    /// * `probed` - The probed audio source.
    /// * `stretch` - The amount of stretching (`new_length / old_length`). A value of `1.0` is no
    /// change, a value less than `1.0` will increase the pitch & decrease the length, and a value
    /// greater than `1.0` will decrease the pitch & increase the length. If a `target_sample_rate`
    /// is given, then the final amount will automatically be adjusted to account for that.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. If this is `None`, then the file will not be resampled and it will stay its
    /// original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// * `resample_quality` - The quality of the resampler to use if the `target_sample_rate`
    /// doesn't match the source sample rate.
    ///     - Has no effect if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `DecodedAudio`
    /// resource can  be in RAM. If the resulting resource is larger than this, then an error
    /// will be returned instead. This is useful to avoid locking up or crashing the system
    /// if the use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    #[cfg(feature = "stretch-sinc-resampler")]
    pub fn decode_probed_stretched(
        &mut self,
        probed: ProbedAudioSource,
        stretch: f64,
        target_sample_rate: Option<NonZeroU32>,
        max_bytes: Option<usize>,
    ) -> Result<DecodedAudioF32, LoadError> {
        decode_f32_stretched(
            probed,
            stretch,
            self.codec_registry,
            max_bytes,
            target_sample_rate,
        )
    }
}

/// An audio source which has had its metadata probed, but has not been decoded yet.
pub struct ProbedAudioSource {
    probed: ProbeResult,
    sample_rate: Option<NonZeroU32>,
    num_channels: NonZeroUsize,
}

impl ProbedAudioSource {
    pub fn probe_result(&self) -> &ProbeResult {
        &self.probed
    }

    pub fn probe_result_mut(&mut self) -> &mut ProbeResult {
        &mut self.probed
    }

    /// The sample rate of the audio source.
    ///
    /// Returns `None` if the sample rate is unkown.
    pub fn sample_rate(&self) -> Option<NonZeroU32> {
        self.sample_rate
    }

    /// Override the sample rate of this resource with the given sample rate. This
    /// can be useful if [`ProbedAudioSource::sample_rate`] returns `None`, but you
    /// know what the sample rate is ahead of time.
    pub fn override_sample_rate(&mut self, sample_rate: NonZeroU32) {
        self.sample_rate = Some(sample_rate);
    }

    pub fn num_channels(&self) -> NonZeroUsize {
        self.num_channels
    }
}

fn decode<'a>(
    mut probed: ProbedAudioSource,
    codec_registry: &'static CodecRegistry,
    max_bytes: Option<usize>,
    #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
    #[cfg(feature = "resampler")] get_resampler: impl FnOnce(
        ResamplerParams,
    )
        -> &'a mut fixed_resample::FixedResampler<
        f32,
        MAX_CHANNELS,
    >,
) -> Result<DecodedAudio, LoadError> {
    let original_sample_rate = probed.sample_rate.unwrap_or_else(|| {
        #[cfg(any(feature = "tracing", feature = "log"))]
        warn!("Audio resource has an unkown sample rate. Assuming a sample rate of 44100...");
        NonZeroU32::new(44100).unwrap()
    });

    #[cfg(feature = "resampler")]
    if let Some(target_sample_rate) = target_sample_rate {
        if original_sample_rate != target_sample_rate {
            // Resampling is needed.
            return resample(
                probed,
                codec_registry,
                max_bytes,
                target_sample_rate,
                original_sample_rate,
                get_resampler,
            )
            .map(|pcm| pcm.into());
        }
    }

    let pcm = decode::decode_native_bitdepth(
        &mut probed.probed,
        probed.num_channels,
        codec_registry,
        original_sample_rate,
        original_sample_rate,
        max_bytes.unwrap_or(DEFAULT_MAX_BYTES),
    )?;

    Ok(pcm)
}

fn decode_f32<'a>(
    mut probed: ProbedAudioSource,
    codec_registry: &'static CodecRegistry,
    max_bytes: Option<usize>,
    #[cfg(feature = "resampler")] target_sample_rate: Option<NonZeroU32>,
    #[cfg(feature = "resampler")] get_resampler: impl FnOnce(
        ResamplerParams,
    )
        -> &'a mut fixed_resample::FixedResampler<
        f32,
        MAX_CHANNELS,
    >,
) -> Result<DecodedAudioF32, LoadError> {
    let original_sample_rate = probed.sample_rate.unwrap_or_else(|| {
        #[cfg(any(feature = "tracing", feature = "log"))]
        warn!("Audio resource has an unkown sample rate. Assuming a sample rate of 44100...");
        NonZeroU32::new(44100).unwrap()
    });

    #[cfg(feature = "resampler")]
    if let Some(target_sample_rate) = target_sample_rate {
        if original_sample_rate != target_sample_rate {
            // Resampling is needed.
            return resample(
                probed,
                codec_registry,
                max_bytes,
                target_sample_rate,
                original_sample_rate,
                get_resampler,
            );
        }
    }

    let pcm = decode::decode_f32(
        &mut probed.probed,
        probed.num_channels,
        codec_registry,
        original_sample_rate,
        original_sample_rate,
        max_bytes.unwrap_or(DEFAULT_MAX_BYTES),
    )?;

    Ok(pcm)
}

#[cfg(feature = "resampler")]
fn resample<'a>(
    mut source: ProbedAudioSource,
    codec_registry: &'static CodecRegistry,
    max_bytes: Option<usize>,
    target_sample_rate: NonZeroU32,
    original_sample_rate: NonZeroU32,
    get_resampler: impl FnOnce(
        ResamplerParams,
    ) -> &'a mut fixed_resample::FixedResampler<f32, MAX_CHANNELS>,
) -> Result<DecodedAudioF32, LoadError> {
    let resampler = get_resampler(ResamplerParams {
        num_channels: source.num_channels,
        source_sample_rate: original_sample_rate,
        target_sample_rate,
    });

    if resampler.num_channels() != source.num_channels {
        return Err(LoadError::InvalidResampler {
            needed_channels: source.num_channels.get(),
            got_channels: resampler.num_channels().get(),
        });
    }

    decode::decode_resampled(
        &mut source.probed,
        codec_registry,
        target_sample_rate,
        original_sample_rate,
        source.num_channels,
        resampler,
        max_bytes.unwrap_or(DEFAULT_MAX_BYTES),
    )
}

#[cfg(feature = "stretch-sinc-resampler")]
fn decode_f32_stretched(
    mut probed: ProbedAudioSource,
    stretch: f64,
    codec_registry: &'static CodecRegistry,
    max_bytes: Option<usize>,
    target_sample_rate: Option<NonZeroU32>,
) -> Result<DecodedAudioF32, LoadError> {
    use fixed_resample::FixedResampler;

    let original_sample_rate = probed.sample_rate.unwrap_or_else(|| {
        #[cfg(any(feature = "tracing", feature = "log"))]
        warn!("Audio resource has an unkown sample rate. Assuming a sample rate of 44100...");
        NonZeroU32::new(44100).unwrap()
    });

    let mut needs_resample = stretch != 1.0;
    if !needs_resample {
        if let Some(target_sample_rate) = target_sample_rate {
            needs_resample = original_sample_rate != target_sample_rate;
        }
    }

    if needs_resample {
        let out_sample_rate = target_sample_rate.unwrap_or(original_sample_rate);
        let ratio = (out_sample_rate.get() as f64 / original_sample_rate.get() as f64) * stretch;

        let mut resampler = FixedResampler::<f32, MAX_CHANNELS>::arbitrary_ratio_sinc(
            original_sample_rate.get(),
            ratio,
            probed.num_channels,
            false,
        );

        if resampler.num_channels() != probed.num_channels {
            return Err(LoadError::InvalidResampler {
                needed_channels: probed.num_channels.get(),
                got_channels: resampler.num_channels().get(),
            });
        }

        return decode::decode_resampled(
            &mut probed.probed,
            codec_registry,
            out_sample_rate,
            original_sample_rate,
            probed.num_channels,
            &mut resampler,
            max_bytes.unwrap_or(DEFAULT_MAX_BYTES),
        );
    }

    decode::decode_f32(
        &mut probed.probed,
        probed.num_channels,
        codec_registry,
        original_sample_rate,
        original_sample_rate,
        max_bytes.unwrap_or(DEFAULT_MAX_BYTES),
    )
}
