#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    name: String,
    class: Option<String>,
    process: Option<String>,
}

impl Window {
    pub fn new<S: Into<String>>(name: S, class: Option<String>, process: Option<String>) -> Self {
        Self {
            name: name.into(),
            class,
            process,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn class(&self) -> Option<&String> {
        self.class.as_ref()
    }
    pub fn process(&self) -> Option<&String> {
        self.process.as_ref()
    }

    pub fn get_libobs_window_id(&self) -> String {
        let mut window_id = String::new();
        window_id.push_str(&self.name);
        window_id.push(':');
        if let Some(class) = &self.class {
            window_id.push_str(class);
        }
        window_id.push(':');
        if let Some(process) = &self.process {
            window_id.push_str(process);
        }
        return window_id;
    }
}

#[cfg(target_os = "windows")]
pub mod window_size {
    use windows::{
        core::PCSTR,
        Win32::{
            Foundation::RECT,
            UI::WindowsAndMessaging::{FindWindowA, GetClientRect},
        },
    };

    use crate::resolution::Size;

    pub fn get_window_size<S: Into<String>>(
        window_title: S,
        window_class: Option<&String>,
    ) -> Result<Size, ()> {
        let mut window_title = window_title.into().clone();
        window_title.push('\0'); // null terminate

        let title = PCSTR(window_title.as_ptr());
        let class = if let Some(cn) = window_class {
            let mut class_name = cn.to_owned();
            class_name.push('\0'); // null terminate
            PCSTR(class_name.as_ptr())
        } else {
            let class_name: PCSTR = PCSTR::default(); // null
            class_name
        };

        let hwnd = unsafe { FindWindowA(class, title) };
        if hwnd.is_invalid() {
            return Err(());
        }

        let mut rect = RECT::default();
        let ok = unsafe { GetClientRect(hwnd, &mut rect as _).as_bool() };
        if ok && rect.right > 0 && rect.bottom > 0 {
            Ok(Size::new(rect.right as u32, rect.bottom as u32))
        } else {
            Err(())
        }
    }
}

#[cfg(target_os = "linux")]
pub mod WindowSize {
    pub fn get_window_size<S: Into<String>>(
        window_title: S,
        window_class: Option<&String>,
    ) -> Result<Size, ()> {
        todo!()
    }
}

#[cfg(target_os = "macos")]
pub mod WindowSize {
    pub fn get_window_size<S: Into<String>>(
        window_title: S,
        window_class: Option<&String>,
    ) -> Result<Size, ()> {
        todo!()
    }
}
