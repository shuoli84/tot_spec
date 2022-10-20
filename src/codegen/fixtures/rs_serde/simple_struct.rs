/// A test struct with different kinds of fields
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SimpleStruct {
    /// required bool field
    pub bool_value: bool,
    /// required i8 field
    pub i8_value: i8,
    pub i64_value: std::option::Option<i64>,
    pub string_value: std::option::Option<std::string::String>,
    pub bytes_value: std::option::Option<std::vec::Vec<u8>>,
    /// string map with customized Map type
    pub string_map:
        std::option::Option<std::collections::BTreeMap<std::string::String, std::string::String>>,
    pub key_values: std::option::Option<KeyValue>,
    pub children: std::option::Option<std::vec::Vec<SimpleStruct>>,
}
