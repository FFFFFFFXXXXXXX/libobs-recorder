use std::ffi::CString;

/*
 Store all the Cstrings in a Vec<CString> until they have been used.
 This is a struct for concisely creating c_strings for use with libobs

 On Drop all the CStrings get dropped automatically.
*/
pub struct Get {
    #[allow(unused)]
    c_strings: Vec<CString>,
}

impl Get {
    pub fn new() -> Self {
        Self { c_strings: Vec::new() }
    }

    /// panics if `name` contains a character with the character code zero (0)
    pub fn c_str(&mut self, string: impl Into<String>) -> *const i8 {
        let s = CString::new(string.into()).unwrap();
        let ptr = s.as_ptr();
        self.c_strings.push(s);
        ptr
    }
}
