#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Framerate(u32);

impl Framerate {
    pub fn new(framerate: u32) -> Self {
        Self(framerate)
    }
    pub fn default() -> Self {
        Framerate(30)
    }
    pub fn is_default(&self) -> bool {
        self.0 == 30
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
