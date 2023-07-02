use crate::Definition;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Context stores use info for a codegen pass
pub struct Context {
    /// All loaded definitions
    definitions: Mutex<HashMap<PathBuf, Arc<Definition>>>,

    /// The path for working definition
    working_definition_path: PathBuf,
}

impl Context {
    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        Ok(Self {
            definitions: Default::default(),
            working_definition_path: path.to_owned(),
        })
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

    pub fn get_working_def_path(&self) -> PathBuf {
        self.working_definition_path.clone()
    }

    /// get the path for namespace
    pub fn get_include_path(&self, namespace: &str, def: &Definition) -> anyhow::Result<PathBuf> {
        let include = def
            .get_include(namespace)
            .ok_or_else(|| anyhow::anyhow!("{} not found", namespace))?;

        let relative_path = &include.path;
        let included_def_path = self
            .working_definition_path
            .parent()
            .unwrap()
            .join(relative_path);
        Ok(included_def_path)
    }

    pub fn load_include_def(
        &self,
        namespace: &str,
        def: &Definition,
    ) -> anyhow::Result<Arc<Definition>> {
        let path = self.get_include_path(namespace, def)?;
        self.load_from_yaml(path)
    }
}
