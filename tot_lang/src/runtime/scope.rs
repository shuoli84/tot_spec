use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Scope {
    /// variables for this scope
    pub(crate) vars: HashMap<String, Value>,
}

impl Scope {
    pub fn insert(&mut self, name: String, value: Value) -> Option<Value> {
        self.vars.insert(name, value)
    }

    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.vars.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }
}
