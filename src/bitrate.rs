#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Bitrate(i64);

impl Bitrate {
    pub fn auto() -> Self {
        Bitrate(0)
    }
    pub fn kbit(kbit: i64) -> Self {
        Bitrate(kbit)
    }
    pub fn mbit(mbit: i64) -> Self {
        Bitrate(mbit * 1000)
    }
    pub fn gbit(gbit: i64) -> Self {
        Bitrate(gbit * 1000000)
    }
    pub fn is_auto(&self) -> bool {
        self.0 > 0
    }
}

impl Into<i64> for Bitrate {
    fn into(self) -> i64 {
        self.0
    }
}
impl From<i64> for Bitrate {
    fn from(bitrate: i64) -> Self {
        Self(bitrate)
    }
}
