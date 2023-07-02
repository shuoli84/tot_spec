use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Parse context
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Definition {
    pub includes: Vec<Include>,
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

    /// load definition from path
    pub fn load_from_yaml(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let def = serde_yaml::from_str::<Definition>(&content)?;
        Ok(def)
    }

    /// get include for namespace
    pub fn get_include(&self, namespace: &str) -> Option<&Include> {
        self.includes
            .iter()
            .filter(|it| it.namespace.eq(namespace))
            .nth(0)
    }
}

/// Include is used to import other definitions.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    /// where to locate the file, it can be relative or absolute
    pub path: String,

    /// namespace is same as use the_def as namespace. So models
    /// in current def can reference with format "namespace.TypeName"
    pub namespace: String,

    /// attributes for Include.
    /// rs_mod is the rs_mod path to use. In codegen, it will become
    ///   "use {rs_mod} as {namespace};"
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ModelDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ModelType,
    /// description of this model
    #[serde(default)]
    pub desc: Option<String>,
    /// attributes for model
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

impl ModelDef {
    /// get attribute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tot_spec::{ModelDef, Type};
    ///
    /// let field = ModelDef::default()
    ///     .with_attribute("test_attr", "attr_value");
    /// assert!(field.attribute("test_attr").is_some());
    /// ```
    pub fn attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    pub fn with_attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(name.into(), value.into());
        self
    }
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
    NewType {
        inner_type: Box<serde_helper::StringOrStruct<Type>>,
    },
    #[serde(rename = "const")]
    Const {
        value_type: ConstType,
        values: Vec<ConstValueDef>,
    },
}

impl Default for ModelType {
    fn default() -> Self {
        Self::Struct(StructDef::default())
    }
}

impl ModelType {
    pub fn new_type(inner_type: Type) -> Self {
        Self::NewType {
            inner_type: Box::new(inner_type.into()),
        }
    }

    /// extract struct_def
    pub fn struct_def(&self) -> Option<&StructDef> {
        match self {
            Self::Struct(ref struct_def) => Some(struct_def),
            _ => None,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    #[serde(default)]
    pub extend: Option<String>,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

impl StructDef {
    /// get field for name
    pub fn field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().filter(|f| f.name.eq(name)).nth(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,

    #[serde(rename = "type")]
    pub type_: serde_helper::StringOrStruct<Type>,

    #[serde(default)]
    pub desc: Option<String>,

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
            desc: None,
            type_: type_.into(),
            attributes: Default::default(),
            required: false,
        }
    }

    /// set field's desc
    pub fn with_desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = Some(desc.into());
        self
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

    /// returns attributes for this field
    pub fn rs_attributes(&self) -> Vec<String> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstValueDef {
    pub name: String,
    pub value: StringOrInteger,
    #[serde(default)]
    pub desc: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConstType {
    I8,
    I16,
    I32,
    I64,
    String,
}

impl ConstType {
    pub fn rs_type(&self) -> &'static str {
        match self {
            ConstType::I8 => "i8",
            ConstType::I16 => "i16",
            ConstType::I32 => "i32",
            ConstType::I64 => "i64",
            ConstType::String => "&'static str",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeReference {
    pub namespace: Option<String>,
    pub target: String,
}

impl TypeReference {
    /// try parse from string
    pub fn try_parse(s: &str) -> Option<Self> {
        if let Some(((namespace, identifier), _)) = serde_helper::if_identifier(s) {
            Some(TypeReference {
                namespace: namespace.map(Into::into),
                target: identifier.to_string(),
            })
        } else {
            None
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
    #[serde(rename = "i16")]
    I16,
    #[serde(rename = "i32")]
    I32,
    #[serde(rename = "i64")]
    I64,
    #[serde(rename = "f64")]
    F64,
    #[serde(rename = "decimal")]
    Decimal,
    #[serde(rename = "bigint")]
    BigInt,
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "list")]
    List {
        item_type: Box<serde_helper::StringOrStruct<Type>>,
    },
    #[serde(rename = "map")]
    Map { value_type: Box<Type> },
    #[serde(rename = "ref")]
    Reference(TypeReference),
    /// json object
    #[serde(rename = "json")]
    Json,
}

impl Type {
    pub fn list(item_type: Type) -> Self {
        Self::List {
            item_type: Box::new(item_type.into()),
        }
    }

    pub fn map(value_type: Type) -> Self {
        Self::Map {
            value_type: value_type.into(),
        }
    }

    pub fn reference(target: impl Into<String>) -> Self {
        Self::Reference(TypeReference {
            namespace: None,
            target: target.into(),
        })
    }
}

impl Type {
    pub fn rs_type(&self) -> String {
        match self {
            Type::Bool => "bool".into(),
            Type::I8 => "i8".into(),
            Type::I16 => "i16".into(),
            Type::I32 => "i32".into(),
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
            Type::Reference(TypeReference {
                namespace: None,
                target,
            }) => target.clone(),
            Type::Reference(TypeReference {
                namespace: Some(namespace),
                target,
            }) => format!("{namespace}::{target}"),
            Type::Json => "serde_json::Value".to_string(),
            Type::Decimal => "rust_decimal::Decimal".into(),
            Type::BigInt => "tot_spec_util::big_int::BigInt".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDef {
    pub name: String,
    pub payload_type: Option<serde_helper::StringOrStruct<Type>>,
    /// also payload can be defined with fields
    #[serde(default)]
    pub payload_fields: Option<Vec<FieldDef>>,
    #[serde(default)]
    pub desc: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StringOrInteger {
    String(String),
    Integer(i64),
}

// code copied from: https://serde.rs/string-or-struct.html
mod serde_helper {
    use super::*;
    use anyhow::bail;
    use serde::{de::Visitor, Deserialize, Deserializer};
    use std::{fmt, marker::PhantomData, ops::Deref};

    #[derive(Debug)]
    pub struct Void {}

    pub trait FromStr: Sized {
        fn from_str(s: &str) -> anyhow::Result<Self>;
    }

    /// parse type, and return rest of str
    fn parse_type(s: &str) -> anyhow::Result<(Type, &str)> {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix("bool") {
            Ok((Type::Bool, rest))
        } else if let Some(rest) = s.strip_prefix("i8") {
            Ok((Type::I8, rest))
        } else if let Some(rest) = s.strip_prefix("i16") {
            Ok((Type::I16, rest))
        } else if let Some(rest) = s.strip_prefix("i32") {
            Ok((Type::I32, rest))
        } else if let Some(rest) = s.strip_prefix("i64") {
            Ok((Type::I64, rest))
        } else if let Some(rest) = s.strip_prefix("f64") {
            Ok((Type::F64, rest))
        } else if let Some(rest) = s.strip_prefix("string") {
            Ok((Type::String, rest))
        } else if let Some(rest) = s.strip_prefix("bytes") {
            Ok((Type::Bytes, rest))
        } else if let Some(rest) = s.strip_prefix("list") {
            // for list,
            let rest_trimmed = rest.trim();
            if let Some(item_type_s) = rest_trimmed.strip_prefix("[") {
                let (item_type, rest) = parse_type(item_type_s)?;
                let rest = rest.trim();
                if let Some(rest) = rest.strip_prefix("]") {
                    Ok((
                        Type::List {
                            item_type: Box::new(item_type.into()),
                        },
                        rest,
                    ))
                } else {
                    bail!(format!("invalid type: {}", s));
                }
            } else {
                bail!(format!("invalid type: {}", s));
            }
        } else if let Some(rest) = s.strip_prefix("map") {
            if let Some(rest) = rest.trim().strip_prefix("[") {
                let (value_type, rest) = parse_type(rest)?;

                if let Some(rest) = rest.strip_prefix("]") {
                    Ok((
                        Type::Map {
                            value_type: Box::new(value_type),
                        },
                        rest,
                    ))
                } else {
                    bail!(format!("invalid type: {}", s));
                }
            } else {
                bail!(format!("invalid type: {}", s));
            }
        } else if let Some(rest) = s.strip_prefix("json") {
            Ok((Type::Json, rest))
        } else if let Some(rest) = s.strip_prefix("decimal") {
            Ok((Type::Decimal, rest))
        } else if let Some(rest) = s.strip_prefix("bigint") {
            Ok((Type::BigInt, rest))
        } else if let Some(((namespace, identifier), rest)) = if_identifier(s) {
            Ok((
                Type::Reference(TypeReference {
                    namespace: namespace.map(Into::into),
                    target: identifier.to_string(),
                }),
                rest,
            ))
        } else {
            bail!("unable to parse: {}", s);
        }
    }

    pub fn if_identifier(s: &str) -> Option<((Option<&str>, &str), &str)> {
        let s = s.trim();
        let mut index: Option<usize> = None;
        for (idx, c) in s.chars().enumerate() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                index = Some(idx);
            } else {
                break;
            }
        }

        if let Some(index) = index {
            // index is the last valid index, split_at index should increase by 1
            let index = index + 1;
            let (name_and_id, rest) = s.split_at(index);
            if let Some((namespace, identifier)) = name_and_id.split_once('.') {
                Some(((Some(namespace), identifier), rest))
            } else {
                Some(((None, name_and_id), rest))
            }
        } else {
            None
        }
    }

    impl serde::Serialize for ConstType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let type_str = match self {
                ConstType::I8 => "i8",
                ConstType::I16 => "i16",
                ConstType::I32 => "i32",
                ConstType::I64 => "i64",
                ConstType::String => "string",
            };

            serializer.serialize_str(type_str)
        }
    }

    impl<'de> serde::Deserialize<'de> for ConstType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Visit;

            impl<'de> serde::de::Visitor<'de> for Visit {
                type Value = ConstType;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("expecting string: i8/i64/string")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match v {
                        "i8" => Ok(ConstType::I8),
                        "i16" => Ok(ConstType::I16),
                        "i32" => Ok(ConstType::I32),
                        "i64" => Ok(ConstType::I64),
                        "string" => Ok(ConstType::String),
                        _ => Err(E::custom(format!("invalid value: {}", v))),
                    }
                }
            }

            deserializer.deserialize_str(Visit)
        }
    }

    impl FromStr for Type {
        fn from_str(s: &str) -> anyhow::Result<Self> {
            let (type_, rest) = parse_type(s)?;
            if !rest.trim().is_empty() {
                bail!("invalid type: {}", s);
            } else {
                Ok(type_)
            }
        }
    }

    /// A wrapper struct which enables parse from string or struct behavior
    /// requires T to impl both FromStr and Deserialize
    pub struct StringOrStruct<T>(pub(crate) T);

    impl<T: Clone> Clone for StringOrStruct<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> std::fmt::Debug for StringOrStruct<T>
    where
        T: FromStr + std::fmt::Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_tuple("StringOrStruct").field(&self.0).finish()
        }
    }

    impl<T: FromStr + serde::Serialize> serde::Serialize for StringOrStruct<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            <T as serde::Serialize>::serialize(&self.0, serializer)
        }
    }

    impl<T> Deref for StringOrStruct<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> From<T> for StringOrStruct<T> {
        fn from(value: T) -> Self {
            Self(value)
        }
    }

    mod string_or_struct {
        use super::*;
        pub struct Visit<T>(pub std::marker::PhantomData<fn() -> T>);

        impl<'de, T: FromStr + Deserialize<'de>> Visitor<'de> for Visit<T> {
            type Value = super::StringOrStruct<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringOrStruct(<T as FromStr>::from_str(value).unwrap()))
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                Ok(StringOrStruct(<T as Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?))
            }
        }
    }

    impl<'de, T: FromStr + serde::Deserialize<'de>> serde::Deserialize<'de> for StringOrStruct<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(string_or_struct::Visit(PhantomData))
        }
    }

    impl serde::Serialize for StringOrInteger {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                StringOrInteger::String(str_val) => serializer.serialize_str(str_val),
                StringOrInteger::Integer(i64_val) => serializer.serialize_i64(*i64_val),
            }
        }
    }

    impl<'de> serde::Deserialize<'de> for StringOrInteger {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct StringOrIntegerVisitor;

            impl<'de> Visitor<'de> for StringOrIntegerVisitor {
                type Value = StringOrInteger;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("string or integer")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(StringOrInteger::String(value.to_string()))
                }

                fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(StringOrInteger::Integer(v))
                }

                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(StringOrInteger::Integer(v as i64))
                }
            }

            deserializer.deserialize_any(StringOrIntegerVisitor)
        }
    }

    #[cfg(test)]
    mod test {
        use super::if_identifier;
        #[test]
        fn test_if_identifier() {
            assert_eq!(if_identifier("normal").unwrap(), ((None, "normal"), ""));
            assert_eq!(
                if_identifier("namespace.normal").unwrap(),
                ((Some("namespace"), "normal"), "")
            );
        }
    }
}
