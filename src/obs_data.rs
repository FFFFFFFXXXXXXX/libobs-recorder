use std::ffi::CString;

use libobs_sys::{
    obs_data, obs_data_create, obs_data_release, obs_data_set_bool, obs_data_set_double,
    obs_data_set_int, obs_data_set_string,
};

/*
 Store all the Cstrings in a Vec<CString> until the obs_data has been used.
 This is necessary since libobs does not clone the strings used in obs_data_set_* calls.
 The required strings are only copied by the functions which take obs_data as an argument

 On Drop release obs_data. The Vec<CString> gets dropped automatically.
*/
pub struct ObsData {
    #[allow(unused)]
    c_strings: Vec<CString>,
    obs_data: *mut obs_data,
}

impl ObsData {
    pub fn new() -> Self {
        Self {
            c_strings: Vec::new(),
            obs_data: unsafe { obs_data_create() },
        }
    }

    pub fn get_ptr(&self) -> *mut obs_data {
        self.obs_data
    }

    pub fn set_string<S1: Into<String>, S2: Into<String>>(&mut self, name: S1, value: S2) {
        let n = CString::new(name.into()).unwrap();
        let v = CString::new(value.into()).unwrap();
        unsafe { obs_data_set_string(self.obs_data, n.as_ptr(), v.as_ptr()) };
        self.c_strings.push(n);
        self.c_strings.push(v);
    }

    pub fn set_int<S: Into<String>, I: Into<i64>>(&mut self, name: S, value: I) {
        let n = CString::new(name.into()).unwrap();
        unsafe { obs_data_set_int(self.obs_data, n.as_ptr(), value.into()) };
        self.c_strings.push(n);
    }

    pub fn set_double<S: Into<String>, F: Into<f64>>(&mut self, name: S, value: F) {
        let n = CString::new(name.into()).unwrap();
        unsafe { obs_data_set_double(self.obs_data, n.as_ptr(), value.into()) };
        self.c_strings.push(n);
    }

    pub fn set_bool<S: Into<String>, B: Into<bool>>(&mut self, name: S, value: B) {
        let n = CString::new(name.into()).unwrap();
        unsafe { obs_data_set_bool(self.obs_data, n.as_ptr(), value.into()) };
        self.c_strings.push(n);
    }
}

impl Drop for ObsData {
    fn drop(&mut self) {
        unsafe { obs_data_release(self.obs_data) };
    }
}
