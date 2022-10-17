use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    /// meta can provide keyvalue metadata for codegen
    pub meta: BTreeMap<String, BTreeMap<String, String>>,
    pub models: Vec<ModelDef>,
}

impl Definition {
    pub fn get_model(&self, name: &str) -> Option<&ModelDef> {
        for model in self.models.iter() {
            if model.name.eq(name) {
                return Some(model);
            }
        }
        None
    }

    /// get the attached key value for codegen
    pub fn get_meta(&self, codegen: &str) -> std::borrow::Cow<BTreeMap<String, String>> {
        match self.meta.get(codegen) {
            Some(key_value) => std::borrow::Cow::Borrowed(key_value),
            None => std::borrow::Cow::Owned(Default::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ModelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum ModelType {
    #[serde(rename = "enum")]
    Enum { variants: Vec<VariantDef> },
    #[serde(rename = "struct")]
    Struct(StructDef),
    #[serde(rename = "virtual")]
    Virtual(StructDef),
    #[serde(rename = "new_type")]
    NewType { inner_type: Box<Type> },
}

impl ModelType {
    pub fn new_type(inner_type: Type) -> Self {
        Self::NewType {
            inner_type: inner_type.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    #[serde(default)]
    pub extend: Option<String>,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,

    #[serde(rename = "type")]
    pub type_: Type,

    #[serde(default)]
    pub attributes: BTreeMap<String, String>,

    #[serde(default)]
    /// whether this field is required
    pub required: bool,
}

impl FieldDef {
    /// create a new field def with name and type
    pub fn new(name: impl Into<String>, type_: Type) -> Self {
        Self {
            name: name.into(),
            type_,
            attributes: Default::default(),
            required: false,
        }
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_attribute(
        mut self,
        attr_name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.attributes.insert(attr_name.into(), value.into());

        self
    }

    /// get attribute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tot_spec::{FieldDef, Type};
    ///
    /// let field = FieldDef::new("test_field", Type::I8)
    ///     .with_attribute("test_attr", "attr_value");
    /// assert!(field.attribute("test_attr").is_some());
    /// ```
    pub fn attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    pub fn rs_type(&self) -> String {
        let ty = self
            .attribute("rs_type")
            .map(|s| s.to_string())
            .unwrap_or(self.type_.rs_type());
        if self.required {
            ty
        } else {
            format!("std::option::Option<{}>", ty)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
/// All types supported
pub enum Type {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "i8")]
    I8,
    #[serde(rename = "i64")]
    I64,
    #[serde(rename = "f64")]
    F64,
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "list")]
    List { item_type: Box<Type> },
    #[serde(rename = "map")]
    Map { value_type: Box<Type> },
    #[serde(rename = "ref")]
    Reference { target: String },
}

impl Type {
    pub fn list(item_type: Type) -> Self {
        Self::List {
            item_type: item_type.into(),
        }
    }

    pub fn map(value_type: Type) -> Self {
        Self::Map {
            value_type: value_type.into(),
        }
    }

    pub fn reference(target: impl Into<String>) -> Self {
        Self::Reference {
            target: target.into(),
        }
    }
}

impl Type {
    pub fn rs_type(&self) -> String {
        match self {
            Type::Bool => "bool".into(),
            Type::I8 => "i8".into(),
            Type::I64 => "i64".into(),
            Type::F64 => "f64".into(),
            Type::Bytes => "std::vec::Vec<u8>".into(),
            Type::String => "std::string::String".into(),
            Type::List { item_type } => format!("std::vec::Vec<{}>", item_type.rs_type()),
            Type::Map { value_type } => {
                format!(
                    "std::collections::HashMap<std::string::String, {}>",
                    value_type.rs_type()
                )
            }
            Type::Reference { target } => target.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDef {
    pub name: String,
    pub payload_type: Option<Type>,
}
