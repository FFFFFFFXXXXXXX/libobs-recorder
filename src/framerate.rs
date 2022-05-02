#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Framerate(u32, u32);

impl Framerate {
    pub fn new(num: u32, den: u32) -> Self {
        Self(num, den)
    }
    pub(crate) fn num(&self) -> u32 {
        self.0
    }
    pub(crate) fn den(&self) -> u32 {
        self.1
    }
    pub(crate) fn is_set(&self) -> bool {
        self.0 > 0 && self.1 > 0
    }
}
