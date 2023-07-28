use std::ffi::CString;

use libobs_sys::{
    obs_data, obs_data_create, obs_data_release, obs_data_set_bool, obs_data_set_int, obs_data_set_string,
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

    pub fn as_ptr(&self) -> *mut obs_data {
        self.obs_data
    }

    pub fn set_string(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let n = CString::new(name.into()).unwrap();
        let v = CString::new(value.into()).unwrap();
        unsafe { obs_data_set_string(self.obs_data, n.as_ptr(), v.as_ptr()) };
        self.c_strings.push(n);
        self.c_strings.push(v);
    }

    pub fn set_int(&mut self, name: impl Into<String>, value: impl Into<i64>) {
        let n = CString::new(name.into()).unwrap();
        unsafe { obs_data_set_int(self.obs_data, n.as_ptr(), value.into()) };
        self.c_strings.push(n);
    }

    pub fn set_bool(&mut self, name: impl Into<String>, value: impl Into<bool>) {
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
