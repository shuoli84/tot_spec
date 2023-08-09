use crate::codegen::style::Style;
use crate::codegen::utils::folder_tree::FolderTree;
use crate::{Definition, ModelDef, ModelType, SpecId, SpecInfo, Type, TypeReference};
use anyhow::anyhow;
use indexmap::IndexMap;
use path_absolutize::path_dedot::ParseDot;
use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Context stores use info for a codegen pass
#[derive(Debug)]
pub struct Context {
    /// All loaded definitions
    specs: IndexMap<SpecId, SpecInfo>,

    /// map from relative path to spec_id
    path_to_spec: IndexMap<PathBuf, SpecId>,

    /// loaded folder tree, it can be used to iter all folder and spec files
    folder_tree: FolderTree,

    /// The root folder for the context, all spec's path is relative to this root path
    root_folder: PathBuf,

    /// optional style config
    style: Option<Style>,
}

impl Context {
    pub fn new_from_folder(folder: &PathBuf) -> anyhow::Result<Self> {
        let folder = folder.absolutize().unwrap().as_ref().to_path_buf();
        let folder = &folder;

        let config_value = Self::load_spec_config(folder)?;
        let style = {
            let mut style: Option<Style> = None;
            if let Some(config) = config_value.as_ref() {
                if let Some(style_value) = config.get("style").cloned() {
                    let style_value = serde_json::from_value::<Style>(style_value)?;
                    style = Some(style_value)
                }
            }
            style
        };

        let mut specs = IndexMap::new();
        let mut path_to_spec = IndexMap::new();

        let mut spec_folder = FolderTree::new();

        for entry in WalkDir::new(folder).sort_by_file_name() {
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

            if spec
                .file_stem()
                .map(|s| s.to_string_lossy().eq("spec_config"))
                .unwrap_or_default()
            {
                // skip spec config file
                continue;
            }

            let spec_id = SpecId::new();

            let relative_path = spec.strip_prefix(folder).unwrap();
            spec_folder.insert(relative_path);

            specs.insert(
                spec_id,
                SpecInfo::new(relative_path.to_owned(), Definition::load_from_yaml(&spec)?),
            );
            path_to_spec.insert(relative_path.to_owned(), spec_id);
        }
        let context = Self {
            specs,
            path_to_spec,
            folder_tree: spec_folder,
            root_folder: folder.clone(),
            style,
        };

        // validate
        let mut violations = context.validate_style();
        violations.extend(context.validate_examples());

        if !violations.is_empty() {
            for violation in violations {
                println!("{violation}");
            }
            anyhow::bail!("validation failed");
        }

        Ok(context)
    }

    /// get a ref to spec's root folder
    pub fn root_folder(&self) -> &PathBuf {
        &self.root_folder
    }

    /// get a ref to the `FolderTree`
    pub fn folder_tree(&self) -> &FolderTree {
        &self.folder_tree
    }

    /// helper to load codegen specific config from spec_config.yaml
    pub fn load_codegen_config<T: serde::de::DeserializeOwned>(
        &self,
        key_name: &str,
    ) -> anyhow::Result<Option<T>> {
        let config_file = self.root_folder().join("spec_config.yaml");
        if !config_file.exists() {
            return Ok(None);
        }

        let config_content = std::fs::read_to_string(config_file)
            .map_err(|_| anyhow::anyhow!("not able to read spec_config.yaml from folder"))?;
        let config_value =
            serde_yaml::from_str::<serde_json::Map<String, serde_json::Value>>(&config_content)?;
        let Some(codegen_value) = config_value.get("codegen") else {
            return Ok(None)
        };

        assert!(codegen_value.is_object());

        let Some(config_value) = codegen_value.as_object().unwrap().get(key_name) else {
            return Ok(None)
        };

        let config = serde_json::from_value::<T>(config_value.to_owned())?;

        Ok(Some(config))
    }

    /// get a ref to definition for spec path, the spec should already loaded
    /// panic if path not loaded
    pub fn get_definition(&self, id: SpecId) -> anyhow::Result<&Definition> {
        self.get_spec_info(id)
            .map(|s| s.definition())
            .ok_or_else(|| anyhow!("spec {id:?} not found"))
    }

    /// get a ref to definition for spec path, the spec should already loaded
    /// panic if path not loaded
    pub fn get_definition_by_path(&self, path: impl AsRef<Path>) -> anyhow::Result<&Definition> {
        let path = path.as_ref();
        let path = self.to_relative_path(path)?;
        let spec_id = self
            .spec_for_path(&path)?
            .ok_or_else(|| anyhow!("spec not found {path:?}"))?;
        self.get_definition(spec_id)
    }

    /// get the spec_id for path
    pub fn spec_for_path(&self, path: impl AsRef<Path>) -> anyhow::Result<Option<SpecId>> {
        let path = path.as_ref();
        let path = self.to_relative_path(path)?;
        Ok(self.path_to_spec.get(&path).cloned())
    }

    /// get the path for spec_id
    pub fn path_for_spec(&self, spec_id: SpecId) -> Option<&PathBuf> {
        self.get_spec_info(spec_id).map(|s| s.relative_path())
    }

    fn get_spec_info(&self, spec_id: SpecId) -> Option<&SpecInfo> {
        self.specs.get(&spec_id)
    }

    /// get model def of the type_ref
    pub fn get_model_def_for_reference(
        &self,
        type_ref: &TypeReference,
        spec_id: SpecId,
    ) -> anyhow::Result<&ModelDef> {
        match &type_ref.namespace {
            Some(namespace) => {
                let namespace_spec = self.get_include_path(namespace, spec_id)?;
                let def = self.get_definition_by_path(namespace_spec)?;
                def.get_model(&type_ref.target)
            }
            None => {
                let def = self.get_definition(spec_id)?;
                def.get_model(&type_ref.target)
            }
        }
        .ok_or_else(|| anyhow!("model {:?} not find", type_ref))
    }

    /// get an iterator for all specs
    pub fn iter_specs(&self) -> impl Iterator<Item = (SpecId, &SpecInfo)> {
        self.specs.iter().map(|(spec_id, spec)| (*spec_id, spec))
    }

    /// get the path for namespace
    pub fn get_include_path(&self, namespace: &str, spec_id: SpecId) -> anyhow::Result<PathBuf> {
        let def = self.get_definition(spec_id)?;
        let include = def
            .get_include(namespace)
            .ok_or_else(|| anyhow::anyhow!("{} not found", namespace))?;

        let relative_path = &include.path;
        let spec_path = self.path_for_spec(spec_id).expect("should exists");
        let included_def_path = spec_path.parent().unwrap().join(relative_path);
        Ok(self.to_relative_path(&included_def_path)?)
    }

    /// get the spec_id for namespace
    pub fn include_spec_for_namespace(
        &self,
        from_spec_id: SpecId,
        namespace: &str,
    ) -> anyhow::Result<SpecId> {
        let path = self.get_include_path(namespace, from_spec_id)?;
        self.spec_for_path(&path)?
            .ok_or_else(|| anyhow!("not able to find spec for path {path:?}"))
    }

    pub fn include_def_for_namespace(
        &self,
        spec_id: SpecId,
        namespace: &str,
    ) -> anyhow::Result<&Definition> {
        let spec_id = self.include_spec_for_namespace(spec_id, namespace)?;
        self.get_definition(spec_id)
    }

    fn to_relative_path(&self, path: &Path) -> anyhow::Result<PathBuf> {
        use path_absolutize::*;
        let path = path.parse_dot_from(&self.root_folder)?;
        let path = path.absolutize_virtually(&self.root_folder)?;
        Ok(pathdiff::diff_paths(path, &self.root_folder).unwrap())
    }

    /// validate all definitions against styles, return violations
    fn validate_style(&self) -> Vec<String> {
        let mut violations = vec![];
        let Some(style) = self.style.as_ref() else {
            return violations;
        };

        for (_, spec_info) in self.specs.iter() {
            let spec_path = spec_info.relative_path();
            if style.is_excluded(spec_path) {
                continue;
            }

            for model in spec_info.definition().models.iter() {
                let model_violations = style.validate_model(model);
                if model_violations.is_empty() {
                    continue;
                }

                for model_violation in model_violations {
                    violations.push(format!(
                        "spec:{:?} model:{} {}",
                        spec_path, model.name, model_violation
                    ));
                }
            }
        }
        violations
    }

    fn validate_examples(&self) -> Vec<String> {
        let mut violations = vec![];
        for (spec_id, spec_info) in self.specs.iter() {
            let spec_path = &spec_info.relative_path();
            for model in spec_info.definition().models.iter() {
                violations.extend(
                    self.validate_example_for_model(model, *spec_id)
                        .into_iter()
                        .map(|v| format!("{spec_path:?} {} {v}", model.name)),
                );
            }
        }
        violations
    }

    fn validate_example_for_model(&self, model: &ModelDef, spec: SpecId) -> Vec<String> {
        let mut violations = vec![];

        for example in model.examples.iter() {
            if example.format.ne("json") {
                // only support validate for json
                continue;
            }

            match serde_json::from_str::<serde_json::Value>(&example.value) {
                Err(e) => {
                    violations.push(format!("{} invalid json: {e:?}", model.name));
                }
                Ok(value) => {
                    violations.extend(self.validate_value_for_model_type(
                        &model.type_,
                        &value,
                        spec,
                    ));
                }
            }
        }

        violations
    }

    fn validate_value_for_model_type(
        &self,
        model_type: &ModelType,
        value: &serde_json::Value,
        spec: SpecId,
    ) -> Vec<String> {
        match &model_type {
            ModelType::Enum { .. } => {
                // todo: support example validate for enum
            }
            ModelType::Struct(st_) => {
                let mut violations = vec![];
                // todo: support extend
                for field in &st_.fields {
                    violations.extend(
                        self.validate_value_for_type(
                            &value.get(&field.name).cloned().unwrap_or_default(),
                            &field.type_,
                            field.required,
                            spec,
                        )
                        .into_iter()
                        .map(|v| format!("field:{} {v}", field.name)),
                    );
                }
                return violations;
            }
            ModelType::Virtual(_) => {
                // skip example validate for virtual model
            }
            ModelType::NewType { inner_type } => {
                return self.validate_value_for_type(&value, &inner_type.as_ref().0, true, spec)
            }
            ModelType::Const { .. } => {
                // skip validate for const
            }
        }

        vec![]
    }

    pub fn validate_value_for_type(
        &self,
        value: &serde_json::Value,
        ty_: &Type,
        required: bool,
        spec: SpecId,
    ) -> Vec<String> {
        if !required && value.is_null() {
            return vec![];
        }

        match ty_ {
            Type::Bool => {
                if !value.is_boolean() {
                    return vec![format!("expect bool, got {:?}", value)];
                }
            }
            Type::I8 => {
                if !value.is_i64() || value.as_i64().unwrap() > i8::MAX as i64 {
                    return vec![format!("expect i8, got {:?}", value)];
                }
            }
            Type::I16 => {
                if !value.is_i64() || value.as_i64().unwrap() > i16::MAX as i64 {
                    return vec![format!("expect i16, got {:?}", value)];
                }
            }
            Type::I32 => {
                if !value.is_i64() || value.as_i64().unwrap() > i32::MAX as i64 {
                    return vec![format!("expect i32, got {:?}", value)];
                }
            }
            Type::I64 => {
                if !value.is_i64() {
                    return vec![format!("expect i64, got {:?}", value)];
                }
            }
            Type::F64 => {
                if !value.is_f64() {
                    return vec![format!("expect f64, got {:?}", value)];
                }
            }
            Type::Decimal => {
                if !value.is_string() {
                    return vec![format!("expect decimal in string, got {:?}", value)];
                }
            }
            Type::BigInt => {
                if !value.is_string() {
                    return vec![format!("expect bigint in string, got {:?}", value)];
                }
            }
            Type::Bytes => {
                if !value.is_string() {
                    return vec![format!("expect bytes in string, got {:?}", value)];
                }
            }
            Type::String => {
                if !value.is_string() {
                    return vec![format!("expect string, got {:?}", value)];
                }
            }
            Type::List { item_type } => {
                if !value.is_array() {
                    return vec![format!("expect array, got {:?}", value)];
                }

                let mut violations = vec![];
                for item in value.as_array().unwrap() {
                    violations.extend(self.validate_value_for_type(item, &item_type, true, spec));
                }
                return violations;
            }
            Type::Map { value_type } => {
                if !value.is_object() {
                    return vec![format!("expect object, got {:?}", value)];
                }

                let mut violations = vec![];
                for (_key, item) in value.as_object().unwrap() {
                    violations.extend(self.validate_value_for_type(item, &value_type, true, spec));
                }
                return violations;
            }
            Type::Reference(type_ref) => {
                let model_def = self.get_model_def_for_reference(type_ref, spec).unwrap();
                return self.validate_value_for_model_type(&model_def.type_, value, spec);
            }
            Type::Json => {
                // always valid
            }
        }

        vec![]
    }
}

// private methods
impl Context {
    fn load_spec_config(
        spec_folder: &PathBuf,
    ) -> anyhow::Result<Option<serde_json::Map<String, serde_json::Value>>> {
        let config_file = spec_folder.join("spec_config.yaml");
        if !config_file.exists() {
            return Ok(None);
        }

        let config_content = std::fs::read_to_string(config_file)
            .map_err(|_| anyhow::anyhow!("not able to read spec_config.yaml from folder"))?;
        let config_value =
            serde_yaml::from_str::<serde_json::Map<String, serde_json::Value>>(&config_content)?;
        Ok(Some(config_value))
    }
}
