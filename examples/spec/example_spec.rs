
/// SimpleStruct
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SimpleStruct {
    pub bool_value: bool,
    pub i8_value: i8,
    pub i64_value: std::option::Option<i64>,
    pub string_value: std::option::Option<std::string::String>,
    pub bytes_value: std::option::Option<std::vec::Vec<u8>>,
    pub string_to_string: std::option::Option<std::collections::BTreeMap::<std::string::String, std::string::String>>,
    pub key_values: std::option::Option<KeyValue>,
    pub children_container: std::option::Option<Container>,
    pub children: std::option::Option<std::vec::Vec<SimpleStruct>>,
}

/// KeyValue
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyValue(pub std::collections::HashMap<std::string::String, std::vec::Vec<u8>>);

/// Container
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Container(pub std::vec::Vec<SimpleStruct>);

/// Base
pub trait Base {
    fn request_id(&self) -> &std::option::Option<std::string::String>;
}

/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Number {
    I64(i64),
    F64(f64),
}

/// AddRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddRequest {
    pub request_id: std::option::Option<std::string::String>,
    pub numbers: std::option::Option<std::vec::Vec<Number>>,
}

impl Base for AddRequest {
    fn request_id(&self) -> &std::option::Option<std::string::String> {
        &self.request_id
    }
}

/// ResetRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResetRequest {
    pub request_id: std::option::Option<std::string::String>,
}

impl Base for ResetRequest {
    fn request_id(&self) -> &std::option::Option<std::string::String> {
        &self.request_id
    }
}
