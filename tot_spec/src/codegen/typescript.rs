use crate::{
    codegen::utils::multiline_prefix_with, models::Definition, ConstType, ConstValueDef, FieldDef,
    StringOrInteger, StructDef, Type, TypeReference, VariantDef,
};
use convert_case::Casing;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use super::context::Context;

pub struct TypeScript {
    context: Context,
    config: TypeScriptConfig,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TypeScriptConfig {
    /// Type mapping for decimal types
    decimal_type: Option<String>,
    /// Type mapping for bigint types
    bigint_type: Option<String>,
    /// Type mapping for json types
    json_type: Option<String>,
    /// Whether to use the export keyword (default: true)
    use_export_keyword: Option<bool>,
}

impl TypeScript {
    fn decimal_type(&self) -> String {
        self.config
            .decimal_type
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "string".to_string())
    }

    fn bigint_type(&self) -> String {
        self.config
            .bigint_type
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "bigint".to_string())
    }

    fn json_type(&self) -> String {
        self.config
            .json_type
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn use_export_keyword(&self) -> bool {
        self.config.use_export_keyword.unwrap_or(true)
    }

    fn ts_type(&self, ty: &Type) -> String {
        match ty {
            Type::Bool => "boolean".into(),
            Type::I8 | Type::I16 | Type::I32 | Type::F64 => "number".into(),
            Type::I64 => self.bigint_type(),
            Type::Bytes => "Uint8Array".into(),
            Type::String => "string".into(),
            Type::List { item_type } => {
                format!("{}[]", self.ts_type(item_type))
            }
            Type::Map { value_type } => {
                format!("Record<string, {}>", self.ts_type(value_type))
            }
            Type::Reference(TypeReference {
                namespace: None,
                target,
            }) => to_pascal_case(target),
            Type::Reference(TypeReference {
                namespace: Some(namespace),
                target,
            }) => {
                format!("{}.{}", to_snake_case(namespace), to_pascal_case(target))
            }
            Type::Json => self.json_type(),
            Type::Decimal => self.decimal_type(),
            Type::BigInt => self.bigint_type(),
        }
    }

    fn ts_type_for_field(&self, field: &FieldDef) -> String {
        // Check for ts_type attribute override
        if let Some(ts_type) = field.attribute("ts_type") {
            return ts_type.clone();
        }

        let ty = self.ts_type(&field.type_);

        // Check for ts_optional attribute override
        let force_optional = field
            .attribute("ts_optional")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        if field.required || force_optional {
            ty
        } else {
            format!("{} | undefined", ty)
        }
    }

    fn ts_field_name(&self, field: &FieldDef) -> String {
        // Check for ts_rename attribute override
        if let Some(rename) = field.attribute("ts_rename") {
            return rename.to_string();
        }

        to_camel_case_reserved(&field.name)
    }

    fn render_struct(
        &self,
        model_name: &str,
        struct_def: &StructDef,
        def: &Definition,
    ) -> anyhow::Result<String> {
        let mut result = String::new();

        // Collect fields from virtual base if extended
        let mut fields = Vec::new();
        if let Some(virtual_name) = &struct_def.extend {
            if let Some(base_model) = def.get_model(virtual_name) {
                if let crate::ModelType::Virtual(base_struct) = &base_model.type_ {
                    fields.extend(base_struct.fields.clone());
                }
            }
        }
        fields.extend(struct_def.fields.clone());

        writeln!(
            result,
            "{}",
            self.export_keyword("interface")
        )?;
        writeln!(result, "{} {{", to_pascal_case(model_name))?;

        for field in &fields {
            if let Some(desc) = &field.desc {
                let desc_to_use = if let Some(ts_desc) = field.attribute("ts_description") {
                    ts_desc
                } else {
                    desc
                };
                writeln!(result, "{}", indent(multiline_prefix_with(desc_to_use, "/// "), 1))?;
            }

            let field_name = self.ts_field_name(field);
            let field_type = self.ts_type_for_field(field);
            writeln!(result, "    {}: {};", field_name, field_type)?;
        }

        writeln!(result, "}}")?;

        Ok(result)
    }

    fn render_enum(
        &self,
        model: &crate::ModelDef,
        variants: &[VariantDef],
    ) -> anyhow::Result<String> {
        let mut result = String::new();

        writeln!(
            result,
            "{}",
            self.export_keyword("type")
        )?;
        writeln!(result, "{} =", to_pascal_case(&model.name))?;

        for (idx, variant) in variants.iter().enumerate() {
            if idx == 0 {
                write!(result, "    ")?;
            } else {
                write!(result, "    | ")?;
            }

            if let Some(desc) = &variant.desc {
                writeln!(result, "{}", indent(multiline_prefix_with(desc, "// "), 1))?;
                write!(result, "    ")?;
            }

            write!(result, "{{ __type: \"{}\"", variant.name)?;

            if let Some(payload_type) = &variant.payload_type {
                let payload_ts = self.ts_type(payload_type);
                writeln!(result, ", payload: {} }}", payload_ts)?;
            } else if let Some(fields) = &variant.payload_fields {
                writeln!(result)?;
                for field in fields {
                    let field_name = self.ts_field_name(field);
                    let field_type = self.ts_type_for_field(field);
                    if let Some(desc) = &field.desc {
                        writeln!(result, "{}", indent(multiline_prefix_with(desc, "/// "), 2))?;
                    }
                    writeln!(result, "        {}: {};", field_name, field_type)?;
                }
                writeln!(result, "    }}")?;
            } else {
                writeln!(result, " }}")?;
            }

            if idx < variants.len() - 1 {
                writeln!(result, "    |")?;
            }
        }

        writeln!(result, ";")?;

        Ok(result)
    }

    fn render_virtual(&self, model_name: &str, struct_def: &StructDef) -> String {
        let mut result = String::new();

        writeln!(
            result,
            "{}",
            self.export_keyword("interface")
        )
        .unwrap();
        writeln!(result, "{} {{", to_pascal_case(model_name)).unwrap();

        for field in &struct_def.fields {
            if let Some(desc) = &field.desc {
                writeln!(
                    result,
                    "{}",
                    indent(multiline_prefix_with(desc, "/// "), 1)
                )
                .unwrap();
            }

            let field_name = self.ts_field_name(field);
            let field_type = self.ts_type_for_field(field);
            writeln!(result, "    {}: {};", field_name, field_type).unwrap();
        }

        writeln!(result, "}}").unwrap();

        result
    }

    fn render_new_type(&self, model_name: &str, inner_type: &Type) -> String {
        let mut result = String::new();

        let inner_ts = self.ts_type(inner_type);
        writeln!(
            result,
            "{}",
            self.export_keyword("type")
        )
        .unwrap();
        writeln!(result, "{} = {};", to_pascal_case(model_name), inner_ts).unwrap();

        result
    }

    fn render_const(
        &self,
        model_name: &str,
        _value_type: &ConstType,
        values: &[ConstValueDef],
    ) -> String {
        let mut result = String::new();

        writeln!(
            result,
            "{}",
            self.export_keyword("type")
        )
        .unwrap();
        write!(result, "{} = ", to_pascal_case(model_name)).unwrap();

        for (idx, value) in values.iter().enumerate() {
            if idx > 0 {
                write!(result, " | ").unwrap();
            }

            let value_literal = ts_const_literal(&value.value);
            write!(result, "{}", value_literal).unwrap();
        }

        writeln!(result, ";").unwrap();

        result
    }

    fn render(&self, spec_path: &Path) -> anyhow::Result<String> {
        let def = self.context.get_definition(spec_path)?;

        let mut result = String::new();

        // Generate import statements
        for include in def.includes.iter() {
            let include_path = self.context.get_include_path(&include.namespace, spec_path)?;
            let relative_path = pathdiff::diff_paths(&include_path, spec_path).unwrap();

            // Convert path to TypeScript import path
            let mut import_path = String::new();
            for component in relative_path.components() {
                match component {
                    std::path::Component::ParentDir => {
                        import_path.push_str("../");
                    }
                    std::path::Component::Normal(name) => {
                        import_path.push_str(&name.to_string_lossy());
                        import_path.push_str("/");
                    }
                    _ => {}
                }
            }
            // Remove trailing slash and add .ts extension
            import_path = import_path.trim_end_matches('/').to_string();
            import_path.push_str(".ts");

            writeln!(
                result,
                "import {{ * as {} }} from \"{}\";",
                to_snake_case(&include.namespace),
                import_path.replace('\\', "/")
            )
            .unwrap();
        }

        if !def.includes.is_empty() {
            writeln!(result).unwrap();
        }

        // Generate model definitions
        for model in def.models.iter() {
            writeln!(result).unwrap();

            if let Some(desc) = &model.desc {
                writeln!(result, "{}", multiline_prefix_with(desc, "// ")).unwrap();
            }

            match &model.type_ {
                crate::ModelType::Enum { variants } => {
                    writeln!(result, "{}", self.render_enum(model, variants)?)?;
                }
                crate::ModelType::Struct(struct_def) => {
                    writeln!(
                        result,
                        "{}",
                        self.render_struct(&model.name, struct_def, def)?
                    )?;
                }
                crate::ModelType::Virtual(struct_def) => {
                    writeln!(result, "{}", self.render_virtual(&model.name, struct_def))?;
                }
                crate::ModelType::NewType { inner_type } => {
                    writeln!(result, "{}", self.render_new_type(&model.name, inner_type))?;
                }
                crate::ModelType::Const { value_type, values } => {
                    writeln!(result, "{}", self.render_const(&model.name, value_type, values))?;
                }
            }
        }

        Ok(result)
    }

    fn export_keyword(&self, keyword: &str) -> String {
        if self.use_export_keyword() {
            format!("export {keyword}")
        } else {
            keyword.to_string()
        }
    }
}

impl super::Codegen for TypeScript {
    fn load_from_folder(folder: &PathBuf) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let context = Context::new_from_folder(folder)?;
        let config = context.load_codegen_config::<TypeScriptConfig>("typescript")?;

        Ok(Self {
            context,
            config: config.unwrap_or_default(),
        })
    }

    fn generate_for_folder(&self, _folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        for (spec_path, _) in self.context.iter_specs() {
            let mut output_path = output.join(spec_path);
            output_path.set_extension("ts");

            let parent_folder = output_path.parent().unwrap();
            std::fs::create_dir_all(parent_folder)?;

            let code = self.render(spec_path)?;

            std::fs::write(&output_path, code)?;
            println!("write output to {:?}", output_path);
        }

        Ok(())
    }
}

fn to_pascal_case(name: &str) -> String {
    name.to_case(convert_case::Case::Pascal)
}

fn to_camel_case_reserved(name: &str) -> String {
    let camel = name.to_case(convert_case::Case::Camel);

    // TypeScript reserved words that need escaping
    const RESERVED: &[&str] = &[
        "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete",
        "do", "else", "enum", "export", "extends", "false", "finally", "for", "function",
        "if", "import", "in", "instanceof", "new", "null", "return", "super", "switch", "this",
        "throw", "true", "try", "typeof", "var", "void", "while", "with", "as", "implements",
        "interface", "let", "package", "private", "protected", "public", "static", "yield",
        "any", "boolean", "constructor", "declare", "get", "module", "require", "number", "set",
        "string", "symbol", "type", "from", "of",
    ];

    if RESERVED.contains(&camel.as_str()) {
        format!("{}_", camel)
    } else {
        camel
    }
}

fn to_snake_case(name: &str) -> String {
    name.to_case(convert_case::Case::Snake)
}

fn indent(s: String, level: usize) -> String {
    let indent = "    ".repeat(level);
    s.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ts_const_literal(val: &StringOrInteger) -> String {
    match val {
        StringOrInteger::String(s) => format!("\"{}\"", s),
        StringOrInteger::Integer(i) => i.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::Codegen;

    #[test]
    fn test_render() {
        fn test_def(spec: &Path, code_path: &str) {
            let spec = spec.strip_prefix("src/codegen/fixtures/specs/").unwrap();

            let codegen =
                TypeScript::load_from_folder(&PathBuf::from("src/codegen/fixtures/specs/")).unwrap();

            let rendered = codegen.render(spec).unwrap();

            let code = std::fs::read_to_string(code_path).unwrap();

            #[cfg(not(feature = "test_update_spec"))]
            pretty_assertions::assert_eq!(code.trim(), rendered.trim());

            #[cfg(feature = "test_update_spec")]
            {
                if code.trim() != rendered.trim() {
                    std::fs::write(code_path, rendered).unwrap();
                }
            }
        }

        for (spec, expected) in &[
            (
                "src/codegen/fixtures/specs/simple_struct.yaml",
                "src/codegen/fixtures/typescript/simple_struct.ts",
            ),
            (
                "src/codegen/fixtures/specs/enum.yaml",
                "src/codegen/fixtures/typescript/enum.ts",
            ),
            (
                "src/codegen/fixtures/specs/extend.yaml",
                "src/codegen/fixtures/typescript/extend.ts",
            ),
            (
                "src/codegen/fixtures/specs/const_i8.yaml",
                "src/codegen/fixtures/typescript/const_i8.ts",
            ),
            (
                "src/codegen/fixtures/specs/const_string.yaml",
                "src/codegen/fixtures/typescript/const_string.ts",
            ),
            (
                "src/codegen/fixtures/specs/new_type.yaml",
                "src/codegen/fixtures/typescript/new_type.ts",
            ),
            (
                "src/codegen/fixtures/specs/include_test.yaml",
                "src/codegen/fixtures/typescript/include_test.ts",
            ),
            (
                "src/codegen/fixtures/specs/decimal.yaml",
                "src/codegen/fixtures/typescript/decimal.ts",
            ),
            (
                "src/codegen/fixtures/specs/bigint.yaml",
                "src/codegen/fixtures/typescript/bigint.ts",
            ),
        ] {
            test_def(PathBuf::from(spec).as_path(), expected);
        }
    }
}
