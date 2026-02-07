use std::{
    ffi::{c_int, c_uint, c_void},
    ptr::NonNull,
};

use crate::{Api, DeviceInfo, RtAudioError, RtAudioErrorType, SampleFormat, SessionID};

pub(crate) struct RtAudioWrapper(NonNull<rtaudio_sys::rtaudio>);

#[cfg(all(feature = "log", not(feature = "tracing")))]
use log::{error, warn};
#[cfg(feature = "tracing")]
use tracing::{error, warn};

impl RtAudioWrapper {
    pub fn new(api: Api) -> Result<Self, RtAudioError> {
        // Safety: This is a static function that is always valid.
        let raw = unsafe { rtaudio_sys::rtaudio_create(api.to_raw()) };

        let Some(raw) = NonNull::new(raw) else {
            return Err(RtAudioError {
                type_: RtAudioErrorType::Unkown,
                msg: Some("failed to create RtAudio instance".into()),
            });
        };

        let new_self = Self(raw);
        new_self.check_for_error()?;

        Ok(new_self)
    }

    pub fn check_for_error(&self) -> Result<(), RtAudioError> {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        let raw_type = unsafe { rtaudio_sys::rtaudio_error_type(self.0.as_ptr()) };

        if let Some(type_) = RtAudioErrorType::from_raw(raw_type) {
            // Safety:
            // * `self.0` cannot be null, and this struct is the only owner.
            // * We assume that RtAudio always returns a valid C string.
            let msg = unsafe {
                let raw_s = rtaudio_sys::rtaudio_error(self.0.as_ptr());
                crate::ffi_utils::c_str_ptr_to_string_lossy(raw_s)
            };

            let e = RtAudioError { type_, msg };

            if let RtAudioErrorType::Warning = e.type_ {
                #[cfg(any(feature = "tracing", feature = "log"))]
                warn!("{}", e);

                Ok(())
            } else {
                Err(e)
            }
        } else {
            Ok(())
        }
    }

    pub fn show_warnings(&mut self, show: bool) {
        let show_int: c_int = if show { 1 } else { 0 };

        // Safety: `self.0` cannot be null, and this struct is the only owner.
        unsafe {
            rtaudio_sys::rtaudio_show_warnings(self.0.as_ptr(), show_int);
        }
    }

    pub fn api(&self) -> Api {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        let api_raw = unsafe { rtaudio_sys::rtaudio_current_api(self.0.as_ptr()) };

        Api::from_raw(api_raw).unwrap_or(Api::Unspecified)
    }

    pub fn get_device_session_id_at_index(&self, i: usize) -> Option<SessionID> {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        let session_id = unsafe { rtaudio_sys::rtaudio_get_device_id(self.0.as_ptr(), i as c_int) };

        if session_id <= 0 {
            None
        } else {
            Some(session_id as u32)
        }
    }

    pub fn get_device_info(&self, session_id: SessionID) -> Result<DeviceInfo, RtAudioError> {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        let device_info_raw =
            unsafe { rtaudio_sys::rtaudio_get_device_info(self.0.as_ptr(), session_id as c_uint) };

        self.check_for_error()?;

        Ok(DeviceInfo::from_raw(device_info_raw))
    }

    pub fn num_devices(&self) -> usize {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        let num_devices = unsafe { rtaudio_sys::rtaudio_device_count(self.0.as_ptr()) };
        num_devices.max(0) as usize
    }

    /// Returns the maximum buffer frames
    ///
    /// # Safety:
    /// `userdata` must be valid until after [`RtAudioWrapper::close_stream()`] is called.
    pub unsafe fn open_stream(
        &self,
        mut output_params: Option<rtaudio_sys::rtaudio_stream_parameters_t>,
        mut input_params: Option<rtaudio_sys::rtaudio_stream_parameters_t>,
        sample_format: SampleFormat,
        sample_rate: u32,
        buffer_frames: u32,
        userdata: *mut c_void,
        options: &mut rtaudio_sys::rtaudio_stream_options_t,
        errcb: rtaudio_sys::rtaudio_error_cb_t,
    ) -> Result<u32, RtAudioError> {
        let mut max_frames: c_uint = buffer_frames as c_uint;

        let output_params_ptr: *mut rtaudio_sys::rtaudio_stream_parameters_t =
            if let Some(output_params) = &mut output_params {
                output_params
            } else {
                std::ptr::null_mut()
            };
        let input_params_ptr: *mut rtaudio_sys::rtaudio_stream_parameters_t =
            if let Some(input_params) = &mut input_params {
                input_params
            } else {
                std::ptr::null_mut()
            };

        rtaudio_sys::rtaudio_open_stream(
            self.0.as_ptr(),
            output_params_ptr,
            input_params_ptr,
            sample_format.to_raw(),
            sample_rate as c_uint,
            &mut max_frames,
            Some(crate::stream::raw_data_callback),
            userdata,
            options,
            errcb,
        );

        if let Err(e) = self.check_for_error() {
            self.close_stream();
            Err(e)
        } else {
            Ok(max_frames)
        }
    }

    pub fn close_stream(&self) {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        unsafe { rtaudio_sys::rtaudio_close_stream(self.0.as_ptr()) };

        #[cfg(any(feature = "tracing", feature = "log"))]
        if let Err(e) = self.check_for_error() {
            error!("Error while closing RtAudio stream: {}", e);
        }
    }

    pub fn start_stream(&self) -> Result<(), RtAudioError> {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        unsafe {
            rtaudio_sys::rtaudio_start_stream(self.0.as_ptr());
        }

        if let Err(e) = self.check_for_error() {
            self.stop_stream();
            Err(e)
        } else {
            Ok(())
        }
    }

    pub fn stop_stream(&self) {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        unsafe { rtaudio_sys::rtaudio_stop_stream(self.0.as_ptr()) };

        #[cfg(any(feature = "tracing", feature = "log"))]
        if let Err(e) = self.check_for_error() {
            error!("Error while stopping RtAudio stream: {}", e);
        }
    }

    pub fn stream_latency(&self) -> Option<usize> {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        let latency = unsafe { rtaudio_sys::rtaudio_get_stream_latency(self.0.as_ptr()) };

        if latency >= 0 {
            Some(latency as usize)
        } else {
            None
        }
    }

    pub fn stream_sample_rate(&self) -> Option<u32> {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        let sr = unsafe { rtaudio_sys::rtaudio_get_stream_sample_rate(self.0.as_ptr()) };

        if sr > 0 {
            Some(sr as u32)
        } else {
            None
        }
    }
}

impl Drop for RtAudioWrapper {
    fn drop(&mut self) {
        // Safety: `self.0` cannot be null, and this struct is the only owner.
        unsafe {
            rtaudio_sys::rtaudio_destroy(self.0.as_ptr());
        }
    }
}
