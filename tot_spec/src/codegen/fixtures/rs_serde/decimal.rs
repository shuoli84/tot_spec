/// struct for decimal field
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestDecimal {
    pub value: std::option::Option<rust_decimal::Decimal>,
}
