/// Const def for string
#[derive(
    Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone,
)]
pub struct Code(pub &'static str);

impl Code {
    fn from_value(val: &str) -> Option<Self> {
        match val {
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
    fn to_value(self) -> &'static str {
        self.0
    }
}

impl Code {
    /// Everything is ok
    pub const Ok: Code = Code("ok");
    /// Request is bad
    pub const Error: Code = Code("error");
}
