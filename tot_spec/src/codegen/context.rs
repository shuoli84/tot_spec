use crate::Definition;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Context stores use info for a codegen pass
pub struct Context {
    /// All loaded definitions
    definitions: Mutex<HashMap<PathBuf, Arc<Definition>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            definitions: Mutex::default(),
        }
    }

    pub fn load_from_yaml<'a>(&self, path: impl Into<PathBuf>) -> anyhow::Result<Arc<Definition>> {
        let path = path.into();

        let mut definitions = self.definitions.lock().unwrap();

        if !definitions.contains_key(&path) {
            let def = Definition::load_from_yaml(&path)?;
            definitions.insert(path.clone(), Arc::new(def));
        }

        Ok(definitions.get(&path).unwrap().clone())
    }

    /// get the path for namespace
    pub fn get_include_path(
        &self,
        namespace: &str,
        def: &Definition,
        spec_path: &Path,
    ) -> anyhow::Result<PathBuf> {
        let include = def
            .get_include(namespace)
            .ok_or_else(|| anyhow::anyhow!("{} not found", namespace))?;

        let relative_path = &include.path;
        let included_def_path = spec_path.parent().unwrap().join(relative_path);
        Ok(included_def_path)
    }

    pub fn load_include_def(
        &self,
        namespace: &str,
        def: &Definition,
        spec_path: &Path,
    ) -> anyhow::Result<Arc<Definition>> {
        let path = self.get_include_path(namespace, def, spec_path)?;
        self.load_from_yaml(path)
    }
}
