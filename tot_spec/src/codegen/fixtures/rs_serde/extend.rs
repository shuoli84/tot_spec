/// Base
pub trait Base {
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

impl Base for AddRequest {
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

impl Base for ResetRequest {
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
