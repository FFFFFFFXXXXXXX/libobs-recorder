#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Framerate(u32, u32);

impl Framerate {
    pub fn new(num: u32, den: u32) -> Self {
        Self(num.max(1), den.max(1)) // let both values be >= 1
    }
    pub fn num(&self) -> u32 {
        self.0
    }
    pub fn den(&self) -> u32 {
        self.1
    }
    pub fn is_valid(&self) -> bool {
        self.0 > 0 && self.1 > 0
    }
}
