/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number {
    /// Variant Int64
    Int64(i64),
    /// Variant Float
    Float(f64),
}
