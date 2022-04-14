#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Framerate(u32);

impl Framerate {
    pub fn new(framerate: u32) -> Self {
        Self(framerate)
    }
    pub fn is_set(&self) -> bool {
        self.0 > 0
    }
}

impl Into<u32> for Framerate {
    fn into(self) -> u32 {
        self.0
    }
}
impl From<u32> for Framerate {
    fn from(framerate: u32) -> Self {
        Self(framerate)
    }
}
