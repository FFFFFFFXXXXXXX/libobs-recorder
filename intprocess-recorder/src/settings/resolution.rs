use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Resolution { width, height }
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

impl From<StdResolution> for Resolution {
    fn from(res: StdResolution) -> Self {
        match res {
            StdResolution::_1024x768p => Resolution { width: 1024, height: 768 },
            StdResolution::_1600x1200p => Resolution { width: 1600, height: 1200 },
            StdResolution::_1280x1024p => Resolution { width: 1280, height: 1024 },
            StdResolution::_1280x720p => Resolution { width: 1280, height: 720 },
            StdResolution::_1366x768p => Resolution { width: 1366, height: 768 },
            StdResolution::_1600x900p => Resolution { width: 1600, height: 900 },
            StdResolution::_1920x1080p => Resolution { width: 1920, height: 1080 },
            StdResolution::_2560x1440p => Resolution { width: 2560, height: 1440 },
            StdResolution::_3840x2160p => Resolution { width: 3840, height: 2160 },
            StdResolution::_5120x2880p => Resolution { width: 5120, height: 2880 },
            StdResolution::_1280x800p => Resolution { width: 1280, height: 800 },
            StdResolution::_1440x900p => Resolution { width: 1440, height: 900 },
            StdResolution::_1680x1050p => Resolution { width: 1680, height: 1050 },
            StdResolution::_1920x1200p => Resolution { width: 1920, height: 1200 },
            StdResolution::_2240x1400p => Resolution { width: 2240, height: 1400 },
            StdResolution::_2560x1600p => Resolution { width: 2560, height: 1600 },
            StdResolution::_2560x1080p => Resolution { width: 2560, height: 1080 },
            StdResolution::_5120x2160p => Resolution { width: 5120, height: 2160 },
            StdResolution::_2580x1080p => Resolution { width: 2580, height: 1080 },
            StdResolution::_3440x1440p => Resolution { width: 3440, height: 1440 },
            StdResolution::_3840x1600p => Resolution { width: 3840, height: 1600 },
            StdResolution::_3840x1080p => Resolution { width: 3840, height: 1080 },
            StdResolution::_5120x1440p => Resolution { width: 5120, height: 1440 },
            StdResolution::_3840x1200p => Resolution { width: 3840, height: 1200 },
        }
    }
}

/// most common resolutions for the aspect ratios 4:3, 5:4, 16:9, 16:10, 21:9, 43:18, 24:10, 32:9, 32:10
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum StdResolution {
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

impl StdResolution {
    pub fn closest_std_resolution(window_size: &Resolution) -> Self {
        use std::cmp::Ordering;

        const DEFAULT_RESOLUTIONS_FOR_ASPECT_RATIOS: [(StdResolution, f64); 9] = [
            (StdResolution::_1600x1200p, 4.0 / 3.0),
            (StdResolution::_1280x1024p, 5.0 / 4.0),
            (StdResolution::_1920x1080p, 16.0 / 9.0),
            (StdResolution::_1920x1200p, 16.0 / 10.0),
            (StdResolution::_2560x1080p, 21.0 / 9.0),
            (StdResolution::_2580x1080p, 43.0 / 18.0),
            (StdResolution::_3840x1600p, 24.0 / 10.0),
            (StdResolution::_3840x1080p, 32.0 / 9.0),
            (StdResolution::_3840x1200p, 32.0 / 10.0),
        ];

        let aspect_ratio = f64::from(window_size.width()) / f64::from(window_size.height());
        // sort difference of aspect_ratio to comparison by absolute values => most similar aspect ratio is at index 0
        let mut aspect_ratios =
            DEFAULT_RESOLUTIONS_FOR_ASPECT_RATIOS.map(|(res, ratio)| (res, f64::abs(ratio - aspect_ratio)));
        aspect_ratios.sort_by(|(_, ratio1), (_, ratio2)| ratio1.partial_cmp(ratio2).unwrap_or(Ordering::Equal));
        aspect_ratios.first().unwrap().0
    }
}
