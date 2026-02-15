/// NewType to i64, and derive Ord macros
#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Id(pub i64);
/// DictNewType
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DictNewType(
    pub std::collections::HashMap<std::string::String, std::vec::Vec<u8>>,
);
