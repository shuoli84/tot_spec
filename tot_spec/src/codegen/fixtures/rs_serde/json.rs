/// Test struct for json field
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestJsonStruct {
    pub json_value: std::option::Option<serde_json::Value>,
}
