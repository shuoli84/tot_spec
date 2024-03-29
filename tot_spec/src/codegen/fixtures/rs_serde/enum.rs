/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number {
    /// Variant Int64
    Int64(i64),
    /// Variant Float
    Float(f64),
    RealNumber(RealNumber),
}

/// RealNumber
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RealNumber {
    pub part_0: std::option::Option<f64>,
    pub part_1: std::option::Option<f64>,
}
