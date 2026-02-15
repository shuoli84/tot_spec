/// Const def for string
#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Reason(pub &'static str);
impl Reason {
    pub fn from_value(val: &str) -> Option<Self> {
        match val {
            "ok" => Some(Self::OK),
            "error" => Some(Self::ERROR),
            _ => None,
        }
    }
    pub fn to_value(self) -> &'static str {
        self.0
    }
}
impl Reason {
    /// Everything is ok
    pub const OK: Reason = Reason("ok");
    /// Request is bad
    pub const ERROR: Reason = Reason("error");
}
