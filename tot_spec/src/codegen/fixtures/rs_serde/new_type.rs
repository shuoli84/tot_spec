/// NewType to i64, and derive Ord macros
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub i64);
