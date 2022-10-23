/// Const def for string
#[derive(
    Debug, serde::Serialize, serde::Deserialize, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Reason(pub &'static str);

impl Reason {
    pub fn from_value(val: &str) -> Option<Self> {
        match val {
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
    pub fn to_value(self) -> &'static str {
        self.0
    }
}

impl Reason {
    /// Everything is ok
    pub const Ok: Reason = Reason("ok");
    /// Request is bad
    pub const Error: Reason = Reason("error");
}
