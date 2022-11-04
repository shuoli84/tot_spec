/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number {
    Real(NumberReal),
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NumberReal {
    pub real: i64,
    pub imagine: i64,
}
impl Into<Number> for NumberReal {
    fn into(self) -> Number {
        Number::Real(self)
    }
}

/// Number2 with variant with fields
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number2 {
    Real { pub real: i64, pub imagine: i64 },
}
