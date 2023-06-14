/// Example of simple struct definition
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SimpleStruct {
    /// bool value
    pub bool_value: bool,
    /// i8 value
    pub i8_value: i8,
    pub i16_value: std::option::Option<i8>,
    pub i32_value: std::option::Option<i8>,
    pub i64_value: std::option::Option<i64>,
    pub string_value: std::option::Option<std::string::String>,
    pub bytes_value: std::option::Option<std::vec::Vec<u8>>,
    pub string_to_string: std::option::Option<
        std::collections::HashMap<std::string::String, std::string::String>,
    >,
    /// nested self
    pub children: std::option::Option<std::vec::Vec<SimpleStruct>>,
    /// this field is required
    pub required_str_value: std::string::String,
}
