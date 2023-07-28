#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Window {
    name: String,
    class: Option<String>,
    process: Option<String>,
}

impl Window {
    pub fn new(name: impl Into<String>, class: Option<String>, process: Option<String>) -> Self {
        Self {
            name: name.into(),
            class,
            process,
        }
    }

    pub(crate) fn get_libobs_window_id(&self) -> String {
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
        window_id
    }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum Resolution {
    _480p,
    _720p,
    _1080p,
    _1440p,
    _2160p,
    _4320p,
}

impl Resolution {
    #[must_use]
    pub fn get_size(&self) -> Size {
        match self {
            Resolution::_480p => Size {
                width: 640,
                height: 480,
            },
            Resolution::_720p => Size {
                width: 1280,
                height: 720,
            },
            Resolution::_1080p => Size {
                width: 1920,
                height: 1080,
            },
            Resolution::_1440p => Size {
                width: 2560,
                height: 1440,
            },
            Resolution::_2160p => Size {
                width: 3840,
                height: 2160,
            },
            Resolution::_4320p => Size {
                width: 7680,
                height: 4320,
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Size { width, height }
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}
