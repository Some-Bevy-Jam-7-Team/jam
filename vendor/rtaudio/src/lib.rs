mod buffer;
mod device_info;
mod enums;
mod error;
mod ffi_utils;
mod host;
mod options;
mod stream;
mod wrapper;

pub use buffer::*;
pub use device_info::*;
pub use enums::*;
pub use error::*;
pub use host::*;
pub use options::*;
pub use stream::*;

/// Get the current RtAudio version.
pub fn version() -> String {
    // Safety: We assume this c string pointer to always be valid.
    unsafe {
        let raw_s = rtaudio_sys::rtaudio_version();
        crate::ffi_utils::c_str_ptr_to_string_lossy(raw_s).unwrap_or_else(|| String::from("error"))
    }
}

/// Get the list of APIs compiled into this instance of RtAudio.
pub fn compiled_apis() -> Vec<Api> {
    // Safety: We assume RtAudio reports the correct length, we check
    // for the null case, and we do not free the `raw_list` pointer.
    let raw_apis_slice: &[rtaudio_sys::rtaudio_api_t] = unsafe {
        let num_compiled_apis = rtaudio_sys::rtaudio_get_num_compiled_apis();

        if num_compiled_apis == 0 {
            return Vec::new();
        }

        let raw_list = rtaudio_sys::rtaudio_compiled_api();

        if raw_list.is_null() {
            return Vec::new();
        }

        std::slice::from_raw_parts(raw_list, num_compiled_apis as usize)
    };

    raw_apis_slice
        .iter()
        .filter_map(|raw_api| Api::from_raw(*raw_api))
        .collect()
}
