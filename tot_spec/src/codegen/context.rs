use crate::codegen::spec_folder::FolderTree;
use crate::Definition;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Context stores use info for a codegen pass
pub struct Context {
    /// All loaded definitions
    definitions: HashMap<PathBuf, Definition>,

    folder_tree: FolderTree,

    root_folder: PathBuf,
}

impl Context {
    pub fn new_from_folder(folder: &PathBuf) -> anyhow::Result<Self> {
        let mut definitions = HashMap::new();
        let mut spec_folder = FolderTree::new();

        for entry in WalkDir::new(folder) {
            let entry = entry.unwrap();
            let spec = entry.path();

            if !spec.is_file() {
                continue;
            }
            if !spec
                .extension()
                .map(|ext| ext == "yaml")
                .unwrap_or_default()
            {
                continue;
            }

            let relative_path = spec.strip_prefix(folder).unwrap();
            spec_folder.insert(relative_path);

            let def = Definition::load_from_yaml(&spec)?;
            definitions.insert(relative_path.to_owned(), def);
        }

        Ok(Self {
            definitions,
            folder_tree: spec_folder,
            root_folder: folder.clone(),
        })
    }

    /// get a ref to spec's root folder
    pub fn root_folder(&self) -> &PathBuf {
        &self.root_folder
    }

    /// get a ref to the `FolderTree`
    pub fn folder_tree(&self) -> &FolderTree {
        &self.folder_tree
    }

    /// get a ref to definition for spec path, the spec should already loaded
    /// panic if path not loaded
    pub fn get_definition(&self, path: impl AsRef<Path>) -> anyhow::Result<&Definition> {
        Ok(self.definitions.get(path.as_ref()).unwrap())
    }

    /// get an iterator for all specs
    pub fn iter_specs(&self) -> impl Iterator<Item = (&PathBuf, &Definition)> {
        self.definitions.iter()
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
    ) -> anyhow::Result<&Definition> {
        let path = self.get_include_path(namespace, def, spec_path)?;
        self.get_definition(path)
    }
}
