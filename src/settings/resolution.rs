#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Resolution {
    _480p,
    _720p,
    _1080p,
    _1440p,
    _2160p,
    _4320p,
}

#[cfg(feature = "full")]
impl Resolution {
    pub(crate) fn get_size(&self) -> Size {
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Size { width, height }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
