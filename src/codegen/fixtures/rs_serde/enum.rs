/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number {
    I64(i64),
    F64(f64),
}
