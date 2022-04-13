#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cqp(u32);

impl Cqp {
    pub fn new(cqp: u32) -> Self {
        Self(cqp)
    }
    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl Into<i64> for Cqp {
    fn into(self) -> i64 {
        self.0 as i64
    }
}
