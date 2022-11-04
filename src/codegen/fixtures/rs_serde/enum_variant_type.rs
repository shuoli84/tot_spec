/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number {
    /// Variant Int64
    Int64(NumberInt64),
    /// Variant Float
    Float(NumberFloat),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NumberInt64(i64);

impl Into<Number> for NumberInt64 {
    fn into(self) -> Number {
        Number::Int64(self)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NumberFloat(f64);

impl Into<Number> for NumberFloat {
    fn into(self) -> Number {
        Number::Float(self)
    }
}
