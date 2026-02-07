use std::ffi::{c_char, CStr, CString, NulError};

/// # Safety:
/// `c_str_ptr` must either point to a valid c string or be null.
pub unsafe fn c_str_ptr_to_string_lossy(c_str_ptr: *const i8) -> Option<String> {
    unsafe {
        if c_str_ptr.is_null() {
            None
        } else {
            let s = CStr::from_ptr(c_str_ptr as *const c_char)
                .to_string_lossy()
                .to_string();

            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
    }
}

pub fn str_to_c_str_array<const MAX_LEN: usize>(
    s: &str,
) -> Result<[c_char; MAX_LEN], StrToCStrArrayError> {
    let cs = CString::new(s)?;
    let cs_slice = cs.as_bytes_with_nul();

    // Safety: i8 and u8 have the same size.
    let cs_slice =
        unsafe { std::slice::from_raw_parts(cs_slice.as_ptr() as *const c_char, cs_slice.len()) };

    if cs_slice.len() > MAX_LEN as usize {
        return Err(StrToCStrArrayError::TooLarge {
            len: cs_slice.len(),
            max_len: MAX_LEN as usize,
        });
    }

    let mut c_array: [c_char; MAX_LEN] = [0; MAX_LEN];
    c_array[0..cs_slice.len()].copy_from_slice(&cs_slice);

    Ok(c_array)
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StrToCStrArrayError {
    #[error("Could not convert string to C string array: {0}")]
    NulError(#[from] NulError),
    #[error("Could not convert string to C string array: string has a length of {len} but the array has a maximum length of {max_len}")]
    TooLarge { len: usize, max_len: usize },
}
