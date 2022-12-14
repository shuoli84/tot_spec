/// KeyValue
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyValue(pub std::collections::HashMap<std::string::String, std::vec::Vec<u8>>);
