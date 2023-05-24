/// SimpleStruct
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleStruct {
    pub bool_value: bool,
    pub i8_value: i8,
    pub i64_value: std::option::Option<i64>,
    pub string_value: std::option::Option<std::string::String>,
    pub bytes_value: std::option::Option<std::vec::Vec<u8>>,
    pub string_to_string: std::option::Option<
        std::collections::BTreeMap::<std::string::String, std::string::String>,
    >,
    pub key_values: std::option::Option<KeyValue>,
    pub children_container: std::option::Option<Container>,
    pub children: std::option::Option<std::vec::Vec<SimpleStruct>>,
}

/// KeyValue
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyValue(
    pub std::collections::HashMap<std::string::String, std::vec::Vec<u8>>,
);

/// Container
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Container(pub std::vec::Vec<SimpleStruct>);

/// RealNumber
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RealNumber {
    pub real: std::option::Option<f64>,
    pub imagine: std::option::Option<f64>,
}

/// Number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Number {
    I64(i64),
    F64(f64),
    RealNumber(RealNumber),
}

/// BaseRequest
pub trait BaseRequest {
    fn request_id(&self) -> &std::option::Option<std::string::String>;
    fn set_request_id(
        &mut self,
        value: std::option::Option<std::string::String>,
    ) -> std::option::Option<std::string::String>;
}

/// AddRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddRequest {
    pub request_id: std::option::Option<std::string::String>,
    pub numbers: std::option::Option<std::vec::Vec<Number>>,
}
impl BaseRequest for AddRequest {
    fn request_id(&self) -> &std::option::Option<std::string::String> {
        &self.request_id
    }
    fn set_request_id(
        &mut self,
        value: std::option::Option<std::string::String>,
    ) -> std::option::Option<std::string::String> {
        std::mem::replace(&mut self.request_id, value)
    }
}

/// ResetRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResetRequest {
    pub request_id: std::option::Option<std::string::String>,
}
impl BaseRequest for ResetRequest {
    fn request_id(&self) -> &std::option::Option<std::string::String> {
        &self.request_id
    }
    fn set_request_id(
        &mut self,
        value: std::option::Option<std::string::String>,
    ) -> std::option::Option<std::string::String> {
        std::mem::replace(&mut self.request_id, value)
    }
}

/// ConstInteger
#[derive(
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Copy,
    Clone,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct ConstInteger(pub i64);
impl ConstInteger {
    pub fn from_value(val: i64) -> Option<Self> {
        match val {
            1 => Some(Self::VALUE_1),
            2 => Some(Self::VALUE_2),
            _ => None,
        }
    }
    pub fn to_value(self) -> i64 {
        self.0
    }
}
impl ConstInteger {
    pub const VALUE_1: ConstInteger = ConstInteger(1);
    pub const VALUE_2: ConstInteger = ConstInteger(2);
}
