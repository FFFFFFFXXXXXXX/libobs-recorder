use std::ffi::CString;

/*
 Store all the Cstrings in a Vec<CString> until they have been used.
 This is a struct for concisely creating c_strings for use with libobs

 On Drop the Vec<CString> gets dropped automatically.
*/
pub struct Get {
    #[allow(unused)]
    c_strings: Vec<CString>,
}

impl Get {
    pub fn new() -> Self {
        Self {
            c_strings: Vec::new(),
        }
    }

    pub fn c_str<S: Into<String>>(&mut self, string: S) -> *const i8 {
        let s = CString::new(string.into()).unwrap();
        let ptr = s.as_ptr();
        self.c_strings.push(s);
        return ptr;
    }
}
