use std::os::raw::{c_int, c_uint, c_void};
use std::pin::Pin;
use std::sync::{Mutex, OnceLock};

use crate::error::{RtAudioError, RtAudioErrorType};
use crate::{Buffers, DeviceID, Host, SampleFormat, StreamConfig, StreamFlags, StreamStatus};

#[cfg(all(feature = "log", not(feature = "tracing")))]
use log::warn;
#[cfg(feature = "tracing")]
use tracing::warn;

/// Information about a running RtAudio stream.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamInfo {
    /// The ID/name of the output audio device being used.
    ///
    /// If no output is being used, then this will be `None`.
    pub output_device: Option<DeviceID>,

    /// The ID/name of the input audio device being used.
    ///
    /// If no input is being used, then this will be `None`.
    pub input_device: Option<DeviceID>,

    /// The number of output audio channels.
    pub out_channels: usize,
    /// The number of input audio channels.
    pub in_channels: usize,

    /// The sample format.
    pub sample_format: SampleFormat,
    /// The sample rate.
    pub sample_rate: u32,

    /// The maximum number of frames that can appear in each call
    /// to `AudioCallback::process()`.
    pub max_frames: usize,

    /// Whether or not the buffers are interleaved (false), or
    /// deinterleaved (true).
    pub deinterleaved: bool,

    /// The internal latency in frames.
    ///
    /// If the API does not report latency, this will be `None`.
    pub latency: Option<usize>,

    /// The number of seconds that have elapsed since the stream was started.
    pub stream_time: f64,
}

/// A handle to an opened RtAudio stream.
///
/// When this struct is dropped, the stream will automatically be stopped
/// and closed.
///
/// Only one stream can exist at a time.
pub struct StreamHandle {
    info: StreamInfo,
    host: Option<Host>,
    started: bool,

    cb_context: Pin<Box<CallbackContext>>,
}

impl StreamHandle {
    pub(crate) fn new(
        host: Host,
        config: &StreamConfig,
    ) -> Result<StreamHandle, (Host, RtAudioError)> {
        let mut raw_options = match config.raw_stream_options() {
            Ok(o) => o,
            Err(e) => return Err((host, e)),
        };

        let mut info = StreamInfo {
            sample_format: config.sample_format,
            deinterleaved: config.flags.contains(StreamFlags::NONINTERLEAVED),

            stream_time: 0.0,

            // These values will be overwritten later.
            out_channels: 0,
            in_channels: 0,
            sample_rate: 0,
            max_frames: 0,
            latency: None,
            input_device: None,
            output_device: None,
        };

        let mut cb_context = Box::pin(CallbackContext {
            info: info.clone(),
            cb: Box::new(|_, _, _| {}), // This will be replaced later.
        });

        let cb_context_ptr: *mut CallbackContext = &mut *cb_context;

        let mut out_device_index = None;
        let mut in_device_index = None;

        let output_params = if let Some(d) = &config.output_device {
            let mut session_id = None;
            if let Some(device_id) = &d.device_id {
                if let Some(device_info) = host.find_device(device_id) {
                    session_id = Some(device_info.session_id);
                    out_device_index = Some(device_info.index);
                } else if d.fallback {
                    #[cfg(any(feature = "tracing", feature = "log"))]
                    warn!("Output audio device with id {:?} was not found. Falling back to default device...", device_id);
                } else {
                    return Err((
                        host,
                        RtAudioError {
                            type_: RtAudioErrorType::NoDevicesFound,
                            msg: Some(format!(
                                "Audio output device with id {:?} was not found",
                                device_id
                            )),
                        },
                    ));
                }
            }

            if session_id.is_none() {
                if let Some(i) = host.default_output_device_index() {
                    session_id = Some(host.devices()[i].id.session_id);
                    out_device_index = Some(i);
                } else if d.no_device_fallback {
                    #[cfg(any(feature = "tracing", feature = "log"))]
                    warn!(
                        "No default output audio device was found. No output stream will be started."
                    );
                } else {
                    return Err((
                        host,
                        RtAudioError {
                            type_: RtAudioErrorType::NoDevicesFound,
                            msg: Some(String::from("No default output audio device was found")),
                        },
                    ));
                }
            }

            session_id.map(|session_id| {
                info.out_channels = d.num_channels.map(|c| c as usize).unwrap_or_else(|| {
                    out_device_index
                    .map(|i| {
                        let num_channels = host.devices()[i].output_channels as usize;

                        // On some platforms like ALSA, some devices are incorrectly reported
                        // to have 64 channels. Assume these are stereo devices.
                        if num_channels >= 32 {
                            warn!("Output device is reported to have a large number of channels: {}. Assuming the device is stereo...", num_channels);
                            2
                        } else { num_channels }
                    })
                    .unwrap_or(2)
                });

                rtaudio_sys::rtaudio_stream_parameters {
                    device_id: session_id as c_uint,
                    num_channels: info.out_channels as c_uint,
                    first_channel: d.first_channel as c_uint,
                }
            })
        } else {
            None
        };

        let input_params = if let Some(d) = &config.input_device {
            let mut session_id = None;
            if let Some(device_id) = &d.device_id {
                if let Some(device_info) = host.find_device(device_id) {
                    session_id = Some(device_info.session_id);
                    in_device_index = Some(device_info.index);
                } else if d.fallback {
                    #[cfg(any(feature = "tracing", feature = "log"))]
                    warn!("Input audio device with id {:?} was not found. Falling back to default device...", device_id);
                } else {
                    return Err((
                        host,
                        RtAudioError {
                            type_: RtAudioErrorType::NoDevicesFound,
                            msg: Some(format!(
                                "Audio input device with id {:?} was not found",
                                device_id
                            )),
                        },
                    ));
                }
            }

            if session_id.is_none() {
                if let Some(i) = host.default_input_device_index() {
                    session_id = Some(host.devices()[i].id.session_id);
                    in_device_index = Some(i);
                } else if d.no_device_fallback {
                    #[cfg(any(feature = "tracing", feature = "log"))]
                    warn!(
                        "No default input audio device was found. No input stream will be started."
                    );
                } else {
                    return Err((
                        host,
                        RtAudioError {
                            type_: RtAudioErrorType::NoDevicesFound,
                            msg: Some(String::from("No default input audio device was found")),
                        },
                    ));
                }
            }

            session_id.map(|session_id| {
                info.in_channels = d.num_channels.map(|c| c as usize).unwrap_or_else(|| {
                    in_device_index
                    .map(|i| {
                        let num_channels = host.devices()[i].input_channels as usize;

                        // On some platforms like ALSA, some devices are incorrectly reported
                        // to have 64 channels. Assume these are stereo devices.
                        if num_channels >= 32 {
                            warn!("Input device is reported to have a large number of channels: {}. Assuming the device is stereo...", num_channels);
                            2
                        } else { num_channels }
                    })
                    .unwrap_or(2)
                });

                rtaudio_sys::rtaudio_stream_parameters {
                    device_id: session_id as c_uint,
                    num_channels: info.in_channels as c_uint,
                    first_channel: d.first_channel as c_uint,
                }
            })
        } else {
            None
        };

        if out_device_index.is_none() && in_device_index.is_none() {
            return Err((
                host,
                RtAudioError {
                    type_: RtAudioErrorType::NoDevicesFound,
                    msg: Some(String::from(
                        "No default input or output audio device was found.",
                    )),
                },
            ));
        }

        let use_sample_rate = if let Some(sr) = config.sample_rate {
            sr
        } else {
            out_device_index
                .map(|i| host.devices()[i].preferred_sample_rate)
                .unwrap_or_else(|| {
                    in_device_index
                        .map(|i| host.devices[i].preferred_sample_rate)
                        .unwrap_or(44100)
                })
        };

        dbg!(&output_params);
        dbg!(&input_params);

        // Safety: We have pinned the `cb_context_ptr` pointer in place,
        // `cb_context_ptr` is a member field of this struct, and the stream
        // is automatically stopped when this struct is dropped, so
        // `cb_context_ptr` will always stay valid for the lifetime the stream
        // is open.
        let max_frames = match unsafe {
            host.wrapper.open_stream(
                output_params,
                input_params,
                config.sample_format,
                use_sample_rate,
                config.buffer_frames,
                cb_context_ptr as *mut c_void,
                &mut raw_options,
                Some(raw_error_callback),
            )
        } {
            Ok(max_frames) => max_frames,
            Err(e) => {
                return Err((host, e));
            }
        };

        // Get info about the stream.
        info.max_frames = max_frames as usize;
        info.latency = host.wrapper.stream_latency();

        if let Err(e) = host.wrapper.check_for_error() {
            host.wrapper.close_stream();
            return Err((host, e));
        }

        info.sample_rate = host.wrapper.stream_sample_rate().unwrap_or(0);

        if let Err(e) = host.wrapper.check_for_error() {
            host.wrapper.close_stream();
            return Err((host, e));
        }

        info.output_device = out_device_index.map(|i| host.devices()[i].id.clone());
        info.input_device = in_device_index.map(|i| host.devices()[i].id.clone());

        cb_context.info = info.clone();

        let stream = Self {
            info,
            host: Some(host),
            started: false,
            cb_context,
        };

        Ok(stream)
    }

    /// Information about the stream.
    pub fn info(&self) -> &StreamInfo {
        &self.info
    }

    /// Returns `true` if the stream has been started.
    pub fn has_started(&self) -> bool {
        self.started
    }

    /// Start the stream.
    ///
    /// * `data_callback` - This gets called whenever there are new buffers
    /// to process.
    ///
    /// If an error is returned, then it means that the stream failed to
    /// start.
    ///
    /// # Panics
    /// Panics if the stream has already been started. (Use
    /// [`StreamHandle::has_started`] to check if it has been started already.)
    pub fn start<F>(&mut self, data_callback: F) -> Result<(), RtAudioError>
    where
        F: FnMut(Buffers<'_>, &StreamInfo, StreamStatus) + Send + 'static,
    {
        assert!(!self.started, "RtAudio stream has already been started");

        self.cb_context.cb = Box::new(data_callback);

        self.host.as_ref().unwrap().wrapper.start_stream()?;

        self.started = true;

        Ok(())
    }

    /// Stop the stream.
    ///
    /// This will block the calling thread until the stream is stopped. After
    /// which the `data_callback` passed into `Stream::start()` will be
    /// dropped.
    ///
    /// This does not close the stream.
    pub fn stop(&mut self) {
        if self.started {
            self.host.as_ref().unwrap().wrapper.stop_stream();

            // Drop the user's callback.
            self.cb_context.cb = Box::new(|_, _, _| {});

            self.started = false;
        }
    }

    /// Close the stream.
    ///
    /// If the stream is running, this will stop the stream first. In that
    /// case, this will block the calling thread until the stream is stopped.
    /// After which the `data_callback` passed into `Stream::start()` will be
    /// dropped.
    pub fn close(mut self) -> Host {
        self.stop();

        let host = self.host.take().unwrap();
        host.wrapper.close_stream();

        host
    }
}

impl Drop for StreamHandle {
    fn drop(&mut self) {
        if self.host.is_none() {
            return;
        }

        self.stop();

        let host = self.host.take().unwrap();
        host.wrapper.close_stream();
    }
}

struct CallbackContext {
    info: StreamInfo,
    cb: Box<dyn FnMut(Buffers<'_>, &StreamInfo, StreamStatus) + Send + 'static>,
}

#[no_mangle]
pub(crate) unsafe extern "C" fn raw_data_callback(
    out: *mut c_void,
    in_: *mut c_void,
    frames: c_uint,
    stream_time: f64,
    status: rtaudio_sys::rtaudio_stream_status_t,
    userdata: *mut c_void,
) -> c_int {
    if userdata.is_null() {
        return 2;
    }
    if frames == 0 {
        return 0;
    }

    let cb_context_ptr = userdata as *mut CallbackContext;

    // Safety: We checked that this is not null. We have also
    // pinned this context in place, and it will always be valid for
    // the lifetime that this stream is open.
    let cb_context = unsafe { &mut *cb_context_ptr };

    cb_context.info.stream_time = stream_time;

    // Safety: We assume that the correct amount of data pointed to by
    // `out` and `in_` exist and that they do not overlap. Also this
    // function correctly checks for the null case.
    let buffers = unsafe {
        Buffers::from_raw(
            out,
            in_,
            frames as usize,
            cb_context.info.out_channels,
            cb_context.info.in_channels,
            cb_context.info.sample_format,
        )
    };

    let status = StreamStatus::from_bits_truncate(status);

    (cb_context.cb)(buffers, &cb_context.info, status);

    0
}

/// Set the global error handling callback that will be called whenever there is
/// an error that causes an audio stream to close. When an error is received, all
/// streams from all hosts should be manually closed or dropped.
///
/// Note, RtAudio provides no way to tell which host/stream the error originates
/// from. So prefer to stop all existing streams when an error is received.
pub fn set_error_callback<F: FnMut(RtAudioError) + Send + 'static>(callback: F) {
    let error_cb = ERROR_CB_SINGLETON.get_or_init(|| Mutex::new(ErrorCallbackSingleton::new()));
    let mut cb_lock = error_cb.lock().unwrap();
    cb_lock.cb = Some(Box::new(callback));
}

static ERROR_CB_SINGLETON: OnceLock<Mutex<ErrorCallbackSingleton>> = OnceLock::new();

pub(crate) struct ErrorCallbackSingleton {
    cb: Option<Box<dyn FnMut(RtAudioError) + Send + 'static>>,
}

impl ErrorCallbackSingleton {
    fn new() -> Self {
        Self { cb: None }
    }
}

#[no_mangle]
pub(crate) unsafe extern "C" fn raw_error_callback(
    raw_err: rtaudio_sys::rtaudio_error_t,
    raw_msg: *const ::std::os::raw::c_char,
) {
    if let Some(type_) = RtAudioErrorType::from_raw(raw_err) {
        if type_ == RtAudioErrorType::Warning {
            // Do nothing. While we could print the warning, we could be
            // in the realtime thread so it's better to not do that.
            return;
        }

        let error_cb = ERROR_CB_SINGLETON.get_or_init(|| Mutex::new(ErrorCallbackSingleton::new()));

        if let Ok(mut cb_lock) = error_cb.lock() {
            if let Some(cb) = &mut cb_lock.cb {
                // Safety:
                // * We assume that RtAudio always returns a valid C string.
                let msg = unsafe { crate::ffi_utils::c_str_ptr_to_string_lossy(raw_msg) };

                let e = RtAudioError { type_, msg };

                cb(e);
            }
        }
    }
}
