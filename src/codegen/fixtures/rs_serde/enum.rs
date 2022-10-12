/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Number {
    I64(i64),
    F64(f64),
}
