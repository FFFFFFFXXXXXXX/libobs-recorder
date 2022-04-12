#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    //use super::*;
    use std::ffi::CString;

    #[test]
    fn test() {
        unsafe {
            let test_string = CString::new("THIS IS A TEST!").unwrap();
            super::blog(super::LOG_INFO, test_string.as_ptr() as *const i8);
        }
    }
}
