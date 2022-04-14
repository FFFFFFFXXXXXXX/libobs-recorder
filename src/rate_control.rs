#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cbr(u32);

impl Cbr {
    pub fn kbit(kbit: u32) -> Self {
        Cbr(kbit)
    }
    pub fn mbit(mbit: u32) -> Self {
        Cbr(mbit * 1000)
    }
    pub fn is_set(&self) -> bool {
        self.0 > 0
    }
}

impl Into<i64> for Cbr {
    fn into(self) -> i64 {
        self.0 as i64
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cqp(u32);

impl Cqp {
    pub fn new(cqp: u32) -> Self {
        Self(cqp.min(30))
    }
    pub fn is_set(&self) -> bool {
        self.0 > 0
    }
}

impl Into<i64> for Cqp {
    fn into(self) -> i64 {
        self.0 as i64
    }
}
