/// TestJsonStruct
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestJsonStruct {
    pub json_value: std::option::Option<serde_json::Value>,
}
