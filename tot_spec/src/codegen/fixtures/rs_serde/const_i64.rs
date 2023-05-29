/// Const def for i64
#[derive(
    Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone,
)]
pub struct Code(pub i64);

impl Code {
    pub fn from_value(val: i64) -> Option<Self> {
        match val {
            0 => Some(Self::OK),
            1 => Some(Self::ERROR),
            _ => None,
        }
    }
    pub fn to_value(self) -> i64 {
        self.0
    }
}

impl Code {
    /// Everything is ok
    pub const OK: Code = Code(0);
    /// Request is bad
    pub const ERROR: Code = Code(1);
}
