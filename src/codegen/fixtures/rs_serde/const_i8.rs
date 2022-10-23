/// Const def for i8
#[derive(
    Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone,
)]
pub struct Code(pub i8);

impl Code {
    pub fn from_value(val: i8) -> Option<Self> {
        match val {
            0 => Some(Self::OK),
            1 => Some(Self::ERROR),
            _ => None,
        }
    }
    pub fn to_value(self) -> i8 {
        self.0
    }
}

impl Code {
    /// Everything is ok
    pub const OK: Code = Code(0);
    /// Request is bad
    pub const ERROR: Code = Code(1);
}
