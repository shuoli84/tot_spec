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
        to_camel_case_reserved(&field.name)
    }

    fn render_struct(
        &self,
        model_name: &str,
        struct_def: &StructDef,
        def: &Definition,
        spec_path: &Path,
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

        // Generate JSON type fields (snake_case)
        let json_fields: Vec<(String, String, &FieldDef)> = fields
            .iter()
            .map(|f| {
                let json_name = to_snake_case(&f.name);
                let ts_type = self.ts_type_for_field(f);
                (json_name, ts_type, f)
            })
            .collect();

        // Generate TypeScript class
        let pascal_name = to_pascal_case(model_name);
        writeln!(
            result,
            "{} {} {{",
            self.export_keyword("class"),
            pascal_name
        )?;

        // Fields (camelCase)
        for field in &fields {
            if let Some(desc) = &field.desc {
                let desc_to_use = if let Some(ts_desc) = field.attribute("ts_description") {
                    ts_desc
                } else {
                    desc
                };
                writeln!(
                    result,
                    "{}",
                    indent(multiline_prefix_with(desc_to_use, "/// "), 1)
                )?;
            }

            let field_name = self.ts_field_name(field);
            let field_type = self.ts_type_for_field(field);
            let definite_assignment = if field.required { "!" } else { "" };
            writeln!(
                result,
                "    {}{}: {};",
                field_name, definite_assignment, field_type
            )?;
        }

        // Constructor
        writeln!(result)?;
        writeln!(result, "    constructor(data: Partial<{}>) {{", pascal_name)?;
        writeln!(result, "        Object.assign(this, data);")?;
        writeln!(result, "    }}")?;

        // toJSON method
        writeln!(result)?;
        writeln!(result, "    toJSON(): any {{")?;
        writeln!(result, "        return {{")?;
        for (json_name, ts_type, field) in &json_fields {
            let field_name = self.ts_field_name(field);
            let converted = self.convert_to_json(&field_name, ts_type, field, spec_path);
            writeln!(result, "            {}: {},", json_name, converted)?;
        }
        writeln!(result, "        }};")?;
        writeln!(result, "    }}")?;

        // fromJSON static method
        writeln!(result)?;
        let json_type_name = format!("{}JSON", pascal_name);
        writeln!(result, "    static fromJSON(json: {{")?;
        for (json_name, ts_type, _) in &json_fields {
            writeln!(result, "        {}: {},", json_name, ts_type)?;
        }
        writeln!(result, "    }}): {} {{", pascal_name)?;
        writeln!(result, "        return new {}({{", pascal_name)?;
        for (json_name, _, field) in &json_fields {
            let field_name = self.ts_field_name(field);
            let converted = self.convert_from_json(json_name, field, spec_path);
            writeln!(result, "            {}: {},", field_name, converted)?;
        }
        writeln!(result, "        }});")?;
        writeln!(result, "    }}")?;

        writeln!(result, "}}")?;

        // Export JSON type
        writeln!(result)?;
        writeln!(result, "export type {} = {{", json_type_name)?;
        for (json_name, ts_type, _) in &json_fields {
            writeln!(result, "    {}: {};", json_name, ts_type)?;
        }
        writeln!(result, "}}")?;

        Ok(result)
    }

    fn convert_to_json(
        &self,
        field_name: &str,
        ts_type: &str,
        field: &FieldDef,
        spec_path: &Path,
    ) -> String {
        // Check if it's an optional type
        let is_optional = ts_type.ends_with(" | undefined");

        // Check if this is a nested struct (Reference type)
        if let Type::Reference(tref) = &*field.type_ {
            if self.should_call_to_json(tref, spec_path) {
                if is_optional {
                    return format!("this.{}?.toJSON()", field_name);
                }
                return format!("this.{}.toJSON()", field_name);
            }
            return format!("this.{}", field_name);
        }

        // Check if this is an array of references
        if let Type::List { item_type } = &*field.type_ {
            if let Type::Reference(tref) = &**item_type.as_ref() {
                if self.should_call_to_json(tref, spec_path) {
                    if is_optional {
                        return format!("this.{}?.map((e) => e.toJSON())", field_name);
                    }
                    return format!("this.{}.map((e) => e.toJSON())", field_name);
                }
            }
        }

        // For primitive types, arrays, Record, just return the field
        format!("this.{}", field_name)
    }

    fn should_call_to_json(&self, type_ref: &TypeReference, spec_path: &Path) -> bool {
        if let Ok(model) = self
            .context
            .get_model_def_for_reference(type_ref, spec_path)
        {
            match &model.type_ {
                crate::ModelType::Struct(_) => true,
                crate::ModelType::Enum { .. } => false,
                crate::ModelType::NewType { inner_type } => {
                    let inner = &***inner_type;
                    matches!(inner, Type::Reference(_))
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn should_call_from_json(&self, type_ref: &TypeReference, spec_path: &Path) -> bool {
        if let Ok(model) = self
            .context
            .get_model_def_for_reference(type_ref, spec_path)
        {
            match &model.type_ {
                crate::ModelType::Struct(_) => true,
                crate::ModelType::Enum { .. } => false,
                crate::ModelType::NewType { inner_type } => {
                    let inner = &***inner_type;
                    matches!(inner, Type::Reference(_))
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn get_type_reference(&self, tref: &TypeReference) -> String {
        match &tref.namespace {
            Some(namespace) => format!(
                "{}.{}",
                to_snake_case(namespace),
                to_pascal_case(&tref.target)
            ),
            None => to_pascal_case(&tref.target),
        }
    }

    fn convert_from_json(&self, json_name: &str, field: &FieldDef, spec_path: &Path) -> String {
        let ts_type = self.ts_type_for_field(field);
        let is_optional = ts_type.ends_with(" | undefined");

        // Check if this is a nested struct (Reference type)
        if let Type::Reference(tref) = &*field.type_ {
            let target_type = self.get_type_reference(tref);
            if self.should_call_from_json(tref, spec_path) {
                if is_optional {
                    return format!(
                        "json.{} ? {}.fromJSON(json.{}) : undefined",
                        json_name, target_type, json_name
                    );
                }
                return format!("{}.fromJSON(json.{})", target_type, json_name);
            }
            return format!("json.{}", json_name);
        }

        // Check if this is an array of references
        if let Type::List { item_type } = &*field.type_ {
            if let Type::Reference(tref) = &**item_type.as_ref() {
                let target_type = self.get_type_reference(tref);
                if self.should_call_from_json(tref, spec_path) {
                    if is_optional {
                        return format!(
                            "json.{}?.map((e: any) => {}.fromJSON(e))",
                            json_name, target_type
                        );
                    }
                    return format!(
                        "json.{}.map((e: any) => {}.fromJSON(e))",
                        json_name, target_type
                    );
                }
            }
        }

        // For primitive types, arrays, Record, just return the field
        format!("json.{}", json_name)
    }

    fn render_enum(
        &self,
        model: &crate::ModelDef,
        variants: &[VariantDef],
    ) -> anyhow::Result<String> {
        let mut result = String::new();

        writeln!(
            result,
            "{} {} =",
            self.export_keyword("type"),
            to_pascal_case(&model.name)
        )?;

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
                writeln!(result)?;
            }
        }

        writeln!(result, ";")?;

        Ok(result)
    }

    fn render_virtual(&self, model_name: &str, struct_def: &StructDef) -> String {
        let mut result = String::new();

        writeln!(
            result,
            "{} {} {{",
            self.export_keyword("interface"),
            to_pascal_case(model_name)
        )
        .unwrap();

        for field in &struct_def.fields {
            if let Some(desc) = &field.desc {
                writeln!(result, "{}", indent(multiline_prefix_with(desc, "/// "), 1)).unwrap();
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
            "{} {} = {};",
            self.export_keyword("type"),
            to_pascal_case(model_name),
            inner_ts
        )
        .unwrap();

        result
    }

    fn render_const(
        &self,
        model_name: &str,
        _value_type: &ConstType,
        values: &[ConstValueDef],
    ) -> String {
        let mut result = String::new();

        write!(
            result,
            "{} {} = ",
            self.export_keyword("type"),
            to_pascal_case(model_name)
        )
        .unwrap();

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
            let include_path = self
                .context
                .get_include_path(&include.namespace, spec_path)?;
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
                        self.render_struct(&model.name, struct_def, def, spec_path)?
                    )?;
                }
                crate::ModelType::Virtual(struct_def) => {
                    writeln!(result, "{}", self.render_virtual(&model.name, struct_def))?;
                }
                crate::ModelType::NewType { inner_type } => {
                    writeln!(result, "{}", self.render_new_type(&model.name, inner_type))?;
                }
                crate::ModelType::Const { value_type, values } => {
                    writeln!(
                        result,
                        "{}",
                        self.render_const(&model.name, value_type, values)
                    )?;
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

    const RESERVED: &[&str] = &[
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "new",
        "null",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "as",
        "implements",
        "interface",
        "let",
        "package",
        "private",
        "protected",
        "public",
        "static",
        "yield",
        "any",
        "boolean",
        "constructor",
        "declare",
        "get",
        "module",
        "require",
        "number",
        "set",
        "string",
        "symbol",
        "type",
        "from",
        "of",
    ];

    if RESERVED.contains(&camel.as_str()) {
        format!("{}_", camel)
    } else {
        camel
    }
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if c.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
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
                TypeScript::load_from_folder(&PathBuf::from("src/codegen/fixtures/specs/"))
                    .unwrap();

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
