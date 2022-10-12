/// Base
pub trait Base {
    fn request_id(&self) -> &std::option::Option<std::string::String>;
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
