/// TestRustKeyword
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestRustKeyword {
    #[serde(rename = "fn")]
    pub fn_: std::option::Option<std::string::String>,
    #[serde(rename = "const")]
    pub const_: std::option::Option<i32>,
}
