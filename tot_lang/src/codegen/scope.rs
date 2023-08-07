use crate::codegen::Codegen;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct CodegenScopes {
    scopes: Vec<CodegenScope>,
}

impl Default for CodegenScopes {
    fn default() -> Self {
        Self {
            // create the root scope by default
            scopes: vec![CodegenScope::default()],
        }
    }
}

impl CodegenScopes {
    pub fn current_scope(&self) -> &CodegenScope {
        self.scopes.last().expect("expect at least one scope")
    }

    pub fn current_scope_mut(&mut self) -> &mut CodegenScope {
        self.scopes.last_mut().expect("expect at least one scope")
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(CodegenScope::default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn update_reference(&mut self, name: String, type_path: String) {
        self.current_scope_mut().set_reference(name, type_path);
    }

    pub fn lookup_reference_type(&self, name: &str) -> Option<&String> {
        for scope in self.scopes.iter().rev() {
            match scope.get_reference(name) {
                Some(type_) => return Some(type_),
                None => {
                    continue;
                }
            }
        }

        None
    }
}

#[derive(Default, Debug)]
pub struct CodegenScope {
    /// reference type maps
    reference_types: HashMap<String, String>,
}

impl CodegenScope {
    pub fn set_reference(&mut self, name: String, type_path: String) {
        self.reference_types.insert(name, type_path);
    }

    pub fn get_reference(&self, name: &str) -> Option<&String> {
        self.reference_types.get(name)
    }
}

/// RAII Helper to manage scope push/pop
pub struct DeferedScopeLock<'a> {
    codegen: &'a mut Codegen,
}

impl<'a> Deref for DeferedScopeLock<'a> {
    type Target = Codegen;

    fn deref(&self) -> &Self::Target {
        self.codegen
    }
}

impl<'a> DerefMut for DeferedScopeLock<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.codegen
    }
}

impl<'a> DeferedScopeLock<'a> {
    pub fn new(codegen: &'a mut Codegen) -> Self {
        codegen.scopes.push_scope();
        Self { codegen }
    }
}

impl Drop for DeferedScopeLock<'_> {
    fn drop(&mut self) {
        self.codegen.scopes.pop_scope();
    }
}
