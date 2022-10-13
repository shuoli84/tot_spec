/// SimpleStruct
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SimpleStruct {
    pub bool_value: bool,
    pub i8_value: i8,
    pub i64_value: std::option::Option<i64>,
    pub string_value: std::option::Option<std::string::String>,
    pub bytes_value: std::option::Option<std::vec::Vec<u8>>,
    pub string_map:
        std::option::Option<std::collections::BTreeMap<std::string::String, std::string::String>>,
    pub key_values: std::option::Option<KeyValue>,
    pub children: std::option::Option<std::vec::Vec<SimpleStruct>>,
}
