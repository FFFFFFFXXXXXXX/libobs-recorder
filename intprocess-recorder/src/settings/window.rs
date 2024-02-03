#[cfg_attr(feature = "specta", derive(specta::Type))]
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

/// most common resolutions for the aspect ratios 4:3, 5:4, 16:9, 16:10, 21:9, 43:18, 24:10, 32:9, 32:10
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum Resolution {
    /// 4:3 1024x768p
    #[serde(rename = "1024x768p")]
    _1024x768p,
    /// 4:3 1600x1200p
    #[serde(rename = "1600x1200p")]
    _1600x1200p,

    /// 5:4 1280x1024p
    #[serde(rename = "1280x1024p")]
    _1280x1024p,

    /// 16:9 1280x720p
    #[serde(alias = "720p")]
    #[serde(rename = "1280x720p")]
    _1280x720p,
    /// 16:9 1366x768p
    #[serde(rename = "1366x768p")]
    _1366x768p,
    /// 16:9 1600x900p
    #[serde(rename = "1600x900p")]
    _1600x900p,
    /// 16:9 1920x1080p
    #[serde(alias = "1080p")]
    #[serde(rename = "1920x1080p")]
    _1920x1080p,
    /// 16:9 2560x1440p
    #[serde(alias = "1440p")]
    #[serde(rename = "2560x1440p")]
    _2560x1440p,
    /// 16:9 3840x2160p
    #[serde(alias = "2160p")]
    #[serde(rename = "3840x2160p")]
    _3840x2160p,
    /// 16:9 5120x2880p
    #[serde(rename = "5120x2880p")]
    _5120x2880p,

    /// 16:10 1280x800p
    #[serde(rename = "1280x800p")]
    _1280x800p,
    /// 16:10 1440x900p
    #[serde(rename = "1440x900p")]
    _1440x900p,
    /// 16:10 1680x1050p
    #[serde(rename = "1680x1050p")]
    _1680x1050p,
    /// 16:10 1920x1200p
    #[serde(rename = "1920x1200p")]
    _1920x1200p,
    /// 16:10 2240x1400p
    #[serde(rename = "2240x1400p")]
    _2240x1400p,
    /// 16:10 2560x1600p
    #[serde(rename = "2560x1600p")]
    _2560x1600p,

    /// 21:9 2560x1080p
    #[serde(rename = "2560x1080p")]
    _2560x1080p,
    /// 21:9 5120x2160p
    #[serde(rename = "5120x2160p")]
    _5120x2160p,

    /// 43:18 2580x1080p
    #[serde(rename = "2580x1080p")]
    _2580x1080p,
    /// 43:18 3440x1440p
    #[serde(rename = "3440x1440p")]
    _3440x1440p,

    /// 24:10 3840x1600p
    #[serde(rename = "3840x1600p")]
    _3840x1600p,

    /// 32:9 3840x1080p
    #[serde(rename = "3840x1080p")]
    _3840x1080p,
    /// 32:9 5120x1440p
    #[serde(rename = "5120x1440p")]
    _5120x1440p,

    /// 32:10 3840x1200p
    #[serde(rename = "3840x1200p")]
    _3840x1200p,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
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

impl From<Resolution> for Size {
    fn from(res: Resolution) -> Self {
        match res {
            Resolution::_1024x768p => Size { width: 1024, height: 768 },
            Resolution::_1600x1200p => Size { width: 1600, height: 1200 },
            Resolution::_1280x1024p => Size { width: 1280, height: 1024 },
            Resolution::_1280x720p => Size { width: 1280, height: 720 },
            Resolution::_1366x768p => Size { width: 1366, height: 768 },
            Resolution::_1600x900p => Size { width: 1600, height: 900 },
            Resolution::_1920x1080p => Size { width: 1920, height: 1080 },
            Resolution::_2560x1440p => Size { width: 2560, height: 1440 },
            Resolution::_3840x2160p => Size { width: 3840, height: 2160 },
            Resolution::_5120x2880p => Size { width: 5120, height: 2880 },
            Resolution::_1280x800p => Size { width: 1280, height: 800 },
            Resolution::_1440x900p => Size { width: 1440, height: 900 },
            Resolution::_1680x1050p => Size { width: 1680, height: 1050 },
            Resolution::_1920x1200p => Size { width: 1920, height: 1200 },
            Resolution::_2240x1400p => Size { width: 2240, height: 1400 },
            Resolution::_2560x1600p => Size { width: 2560, height: 1600 },
            Resolution::_2560x1080p => Size { width: 2560, height: 1080 },
            Resolution::_5120x2160p => Size { width: 5120, height: 2160 },
            Resolution::_2580x1080p => Size { width: 2580, height: 1080 },
            Resolution::_3440x1440p => Size { width: 3440, height: 1440 },
            Resolution::_3840x1600p => Size { width: 3840, height: 1600 },
            Resolution::_3840x1080p => Size { width: 3840, height: 1080 },
            Resolution::_5120x1440p => Size { width: 5120, height: 1440 },
            Resolution::_3840x1200p => Size { width: 3840, height: 1200 },
        }
    }
}
