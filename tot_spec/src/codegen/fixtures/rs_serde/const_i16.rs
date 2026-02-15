/// Const def for i16
/// Second line of comment
#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
)]
pub struct Code(pub i16);
impl Code {
    pub fn from_value(val: i16) -> Option<Self> {
        match val {
            0 => Some(Self::OK),
            1 => Some(Self::ERROR),
            _ => None,
        }
    }
    pub fn to_value(self) -> i16 {
        self.0
    }
}
impl Code {
    /// Everything is ok
    pub const OK: Code = Code(0);
    /// Request is bad
    pub const ERROR: Code = Code(1);
}
