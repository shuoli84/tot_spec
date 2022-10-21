/// Const def
#[derive(Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reason(pub i64);

impl Reason {
    fn from_value(val: i64) -> Option<Self> {
        match val {
            200 => Some(Self::Ok),
            400 => Some(Self::BadRequest),
            _ => None,
        }
    }
    fn to_value(self) -> i64 {
        self.0
    }
}

impl Reason {
    /// Everything is ok
    pub const Ok: Reason = Reason(200);
    /// Request is bad
    pub const BadRequest: Reason = Reason(400);
}
