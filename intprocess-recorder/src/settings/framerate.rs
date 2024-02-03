#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Framerate(u32, u32);

impl Framerate {
    #[must_use]
    pub fn new(num: u32, den: u32) -> Self {
        Self(num.max(1), den.max(1)) // let both values be >= 1
    }

    #[must_use]
    pub fn num(&self) -> u32 {
        self.0
    }

    #[must_use]
    pub fn den(&self) -> u32 {
        self.1
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.0 > 0 && self.1 > 0
    }
}
