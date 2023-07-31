use super::scope::*;
use anyhow::*;

/// A function call frame, it is the state for one function
#[derive(Debug)]
pub struct Frame {
    // scope stack, last is the top
    scopes: Vec<Scope>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }
}

impl Frame {
    /// push a new scope to the frame
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    /// pop the scope
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        // should not pop the root scope, the root scope should be dropped with
        // frame
        assert!(!self.scopes.is_empty());
    }

    /// Store to current scope
    pub fn store(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.current_scope_mut().insert(name.into(), value);
    }

    /// Assign, if not found, return error
    pub fn assign(&mut self, name: &str, value: serde_json::Value) -> anyhow::Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            match scope.vars.get_mut(name) {
                None => continue,
                Some(v) => {
                    *v = value;
                    return Ok(());
                }
            }
        }
        bail!("{name} not found");
    }

    /// load value for name recursively, it looks up parent scopes
    pub fn load(&self, name: &str) -> Option<&serde_json::Value> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.vars.get(name) {
                return Some(v);
            }
        }

        None
    }

    /// load the name, if it doesn't exist, then raise an error
    pub fn load_required(&self, name: &str) -> anyhow::Result<&serde_json::Value> {
        self.load(name)
            .ok_or_else(|| anyhow!("local var not found: {name}"))
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }
}
