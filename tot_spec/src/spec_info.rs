use crate::Definition;
use std::path::PathBuf;

/// NewType for spec id, it is unique in one run
#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SpecId(u64);

impl SpecId {
    pub fn new() -> Self {
        use std::sync::atomic::*;
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }
}

#[derive(Debug)]
pub struct SpecInfo {
    /// spec's file path
    path: PathBuf,

    /// type path prefix, in format "part1::part2::part3", it is constructed from path "part1/part2/part3.yaml"
    type_path_prefix: String,

    /// Loaded definition
    definition: Definition,
}

impl SpecInfo {
    /// Create a new SpecInfo
    pub fn new(path: PathBuf, definition: Definition) -> Self {
        let type_path_prefix = Self::create_type_path(&path);

        Self {
            path,
            type_path_prefix,
            definition,
        }
    }

    /// Get a reference to the definition
    pub fn definition(&self) -> &Definition {
        &self.definition
    }

    /// Get type_path_prefix for this spec
    pub fn type_path_prefix(&self) -> &str {
        &self.type_path_prefix
    }

    /// Get the relative path to context root
    pub fn relative_path(&self) -> &PathBuf {
        &self.path
    }

    /// Construct the type_path_prefix from relative path
    fn create_type_path(path: &PathBuf) -> String {
        assert!(path.is_relative());

        let components = path.components().collect::<Vec<_>>();
        components
            .iter()
            .map(|c| match c {
                std::path::Component::Normal(name) => {
                    let name = name.to_string_lossy().to_string();
                    name.strip_suffix(".yaml")
                        .map(|s| s.to_string())
                        .unwrap_or(name)
                }
                _ => {
                    unimplemented!()
                }
            })
            .collect::<Vec<_>>()
            .join("::")
    }
}
