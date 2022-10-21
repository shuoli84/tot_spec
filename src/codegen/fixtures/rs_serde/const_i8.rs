/// Const def for i8
#[derive(Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Code(pub i8);

impl Code {
    fn from_value(val: i8) -> Option<Self> {
        match val {
            0 => Some(Self::Ok),
            1 => Some(Self::Error),
            _ => None,
        }
    }
    fn to_value(self) -> i8 {
        self.0
    }
}

impl Code {
    /// Everything is ok
    pub const Ok: Code = Code(0);
    /// Request is bad
    pub const Error: Code = Code(1);
}
