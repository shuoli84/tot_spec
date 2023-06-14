/// Base
pub trait Base {
    fn id(&self) -> &std::string::String;
    fn set_id(&mut self, value: std::string::String) -> std::string::String;
}
/// Child
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Child {
    pub id: std::string::String,
    pub name: std::string::String,
}
impl Base for Child {
    fn id(&self) -> &std::string::String {
        &self.id
    }
    fn set_id(&mut self, value: std::string::String) -> std::string::String {
        std::mem::replace(&mut self.id, value)
    }
}
