/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number {
    /// Variant I64
    I64(i64),
    /// Variant F64
    F64(f64),
}
