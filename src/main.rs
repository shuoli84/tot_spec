mod render;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::render::render;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    models: Vec<ModelDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDef {
    name: String,
    #[serde(rename = "type")]
    type_: ModelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum ModelType {
    #[serde(rename = "enum")]
    Enum { variants: Vec<VariantDef> },
    #[serde(rename = "struct")]
    Struct(StructDef),
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
    extend: Option<String>,
    fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    name: String,
    #[serde(rename = "type")]
    type_: Type,
    attributes: BTreeMap<String, String>,
    required: bool,
}

impl FieldDef {
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
    /// use dummy_spec::{FieldDef, Type};
    ///
    /// let field = FieldDef::new("test_field", Type::I8)
    ///     .with_attribute("test_attr", "attr_value");
    /// assert!(field.attribute("test_attr").is_some());
    /// ```
    pub fn attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    fn rs_type(&self) -> String {
        self.attribute("rs_type")
            .map(|s| s.to_string())
            .unwrap_or(self.type_.rs_type())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasDef {
    #[serde(rename = "type")]
    type_: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
/// All types supported
pub enum Type {
    #[serde(rename = "unit")]
    Unit,
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
    Map {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
    #[serde(rename = "ref")]
    Reference { target: String },
}

impl Type {
    pub fn list(item_type: Type) -> Self {
        Self::List {
            item_type: item_type.into(),
        }
    }

    pub fn map(key_type: Type, value_type: Type) -> Self {
        Self::Map {
            key_type: key_type.into(),
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
    fn rs_type(&self) -> String {
        match self {
            Type::Unit => "()".into(),
            Type::Bool => "bool".into(),
            Type::I8 => "i8".into(),
            Type::I64 => "i64".into(),
            Type::F64 => "f64".into(),
            Type::Bytes => "std::vec::Vec::<u8>".into(),
            Type::String => "std::string::String".into(),
            Type::List { item_type } => format!("std::vec::Vec::<{}>", item_type.rs_type()),
            Type::Map {
                key_type,
                value_type,
            } => {
                format!(
                    "std::collections::HashMap::<{}, {}>",
                    key_type.rs_type(),
                    value_type.rs_type()
                )
            }
            Type::Reference { target } => target.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDef {
    name: String,
    playload_type: Type,
}

fn main() {
    let def = Definition {
        models: vec![
            ModelDef {
                name: "SimpleStruct".to_string(),
                type_: ModelType::Struct(StructDef {
                    extend: None,
                    fields: vec![
                        FieldDef::new("bool_value", Type::Bool),
                        FieldDef::new("i8_value", Type::I8),
                        FieldDef::new("i64_value", Type::I64),
                        FieldDef::new("string_value", Type::String),
                        FieldDef::new("bytes_value", Type::Bytes),
                        FieldDef::new("i8_to_string", Type::map(Type::I8, Type::String))
                            .with_attribute(
                                "rs_type",
                                "std::collections::BTreeMap::<i8, std::string::String>",
                            ),
                        FieldDef::new("key_values", Type::reference("KeyValue")),
                        FieldDef::new("children", Type::list(Type::reference("SimpleStruct"))),
                    ],
                }),
            },
            ModelDef {
                name: "KeyValue".into(),
                type_: ModelType::new_type(Type::map(Type::String, Type::Bytes)),
            },
            ModelDef {
                name: "Container".into(),
                type_: ModelType::new_type(Type::list(Type::reference("SimpleStruct"))),
            },
            ModelDef {
                name: "Base".into(),
                type_: ModelType::Struct(StructDef {
                    extend: None,
                    fields: vec![FieldDef::new("request_id", Type::String)],
                }),
            },
            ModelDef {
                name: "Number".into(),
                type_: ModelType::Enum {
                    variants: vec![
                        VariantDef {
                            name: "I64".into(),
                            playload_type: Type::I64,
                        },
                        VariantDef {
                            name: "F64".into(),
                            playload_type: Type::F64,
                        },
                    ],
                },
            },
            ModelDef {
                name: "AddRequest".into(),
                type_: ModelType::Struct(StructDef {
                    extend: Some("Base".into()),
                    fields: vec![FieldDef::new(
                        "numbers",
                        Type::list(Type::reference("Number")),
                    )],
                }),
            },
        ],
    };
    println!("{}", render(&def).unwrap());
}
