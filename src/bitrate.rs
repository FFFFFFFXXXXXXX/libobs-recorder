#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Bitrate(u32);

impl Bitrate {
    pub fn kbit(kbit: u32) -> Self {
        Bitrate(kbit)
    }
    pub fn mbit(mbit: u32) -> Self {
        Bitrate(mbit * 1000)
    }
    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl Into<i64> for Bitrate {
    fn into(self) -> i64 {
        self.0 as i64
    }
}
