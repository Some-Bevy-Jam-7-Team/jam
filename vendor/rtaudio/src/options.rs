use rtaudio_sys::MAX_NAME_LENGTH;
use std::os::raw::{c_int, c_uint};

use crate::error::{RtAudioError, RtAudioErrorType};
use crate::{DeviceID, SampleFormat, StreamFlags};

/// Used for specifying the parameters of a device when opening a
/// stream.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceParams {
    /// The ID of the device to use.
    ///
    /// If this is `None`, then the default device will be used.
    ///
    /// The default value is `None`.
    #[cfg_attr(feature = "serde", serde(default = "default_device_id"))]
    pub device_id: Option<DeviceID>,
    /// The number of channels in the device to use.
    ///
    /// Set to `None` to use the default number of channels for the device.
    ///
    /// The default value is `None`.
    #[cfg_attr(feature = "serde", serde(default = "default_num_channels"))]
    pub num_channels: Option<u32>,
    /// The first channel index on the device to use.
    ///
    /// The default value is `0`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub first_channel: u32,
    /// If `true`, then fallback to the default device if the device
    /// with the given ID is not found. Otherwise, don't start the stream
    /// and return an error.
    ///
    /// The default value is `true`.
    #[cfg_attr(feature = "serde", serde(default = "default_fallback"))]
    pub fallback: bool,
    /// If `true`, then fallback to no input/output if no default device
    /// could be found. Otherwise, don't start the stream and return an
    /// error.
    ///
    /// The default value is `true`.
    #[cfg_attr(feature = "serde", serde(default = "default_fallback"))]
    pub no_device_fallback: bool,
}

impl Default for DeviceParams {
    fn default() -> Self {
        Self {
            device_id: default_device_id(),
            num_channels: default_num_channels(),
            first_channel: 0,
            fallback: default_fallback(),
            no_device_fallback: default_fallback(),
        }
    }
}

const fn default_device_id() -> Option<DeviceID> {
    None
}

const fn default_num_channels() -> Option<u32> {
    None
}

const fn default_fallback() -> bool {
    true
}

/// The configuration of an audio stream.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamConfig {
    /// The parameters for the output device to use. If you do
    /// not wish to use an output device, set this to `None`.
    ///
    /// The default value is `Some(Default::default())`.
    #[cfg_attr(feature = "serde", serde(default = "default_output_device"))]
    pub output_device: Option<DeviceParams>,

    /// The parameters for the input device to use. If you do not
    /// wish to use an input device, set this to `None`.
    ///
    /// The default value is `None`.
    #[cfg_attr(feature = "serde", serde(default = "default_input_device"))]
    pub input_device: Option<DeviceParams>,

    /// The sample format to use. If the device doesn't natively
    /// support the given format, then it will automatically be converted to/from
    /// that format.
    ///
    /// The default value is `SampleFormat::Float32`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sample_format: SampleFormat,

    /// The sample rate to use. The stream may decide to use a
    /// different sample rate if it's not supported. Set to `None` to use the
    /// output device's default sample rate.
    ///
    /// The default value is `None`.
    #[cfg_attr(feature = "serde", serde(default = "default_sample_rate"))]
    pub sample_rate: Option<u32>,

    /// The desired maximum number of frames that can appear in a
    /// single process call. The stream may decide to use a different value if it's
    /// not supported. A value of zero can be specified, in which case the lowest
    /// allowable value is determined.
    ///
    /// The default value is `1024`.
    #[cfg_attr(feature = "serde", serde(default = "default_buffer_frames"))]
    pub buffer_frames: u32,

    /// The bit flag parameters for this stream.
    ///
    /// By default, no flags are set.
    #[cfg_attr(feature = "serde", serde(default))]
    pub flags: StreamFlags,

    /// Used to control stream latency in the Windows DirectSound, Linux OSS, and Linux Alsa APIs only.
    /// A value of two is usually the smallest allowed. Larger numbers can potentially result in more
    /// robust stream performance, though likely at the cost of stream latency.
    ///
    /// The actual value used when the stream is ran may be different.
    ///
    /// The default value is `4`.
    #[cfg_attr(feature = "serde", serde(default = "default_num_buffers"))]
    pub num_buffers: u32,

    /// Scheduling priority of callback thread (only used with flag `StreamFlags::SCHEDULE_REALTIME`).
    ///
    /// Use a value of `-1` for the default priority.
    ///
    /// The default value is `-1`.
    #[cfg_attr(feature = "serde", serde(default = "default_priority"))]
    pub priority: i32,

    /// The name of the stream (currently used only in Jack).
    ///
    /// The size of the name cannot exceed 511 bytes.
    #[cfg_attr(feature = "serde", serde(default))]
    pub name: String,
}

impl StreamConfig {
    pub fn raw_stream_options(
        &self,
    ) -> Result<rtaudio_sys::rtaudio_stream_options_t, RtAudioError> {
        let name = crate::ffi_utils::str_to_c_str_array::<{ MAX_NAME_LENGTH }>(&self.name)
            .map_err(|e| RtAudioError {
                type_: RtAudioErrorType::InvalidParamter,
                msg: Some(format!("Stream name is invalid: {}", e)),
            })?;

        Ok(rtaudio_sys::rtaudio_stream_options_t {
            flags: self.flags.bits(),
            num_buffers: self.num_buffers as c_uint,
            priority: self.priority as c_int,
            name,
        })
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            output_device: default_output_device(),
            input_device: default_input_device(),
            sample_format: SampleFormat::default(),
            sample_rate: default_sample_rate(),
            buffer_frames: default_buffer_frames(),
            flags: StreamFlags::empty(),
            num_buffers: default_num_buffers(),
            priority: default_priority(),
            name: String::from("RtAudio-rs Client"),
        }
    }
}

fn default_output_device() -> Option<DeviceParams> {
    Some(Default::default())
}

const fn default_input_device() -> Option<DeviceParams> {
    None
}

const fn default_buffer_frames() -> u32 {
    1024
}

const fn default_sample_rate() -> Option<u32> {
    None
}

const fn default_num_buffers() -> u32 {
    4
}

const fn default_priority() -> i32 {
    -1
}
