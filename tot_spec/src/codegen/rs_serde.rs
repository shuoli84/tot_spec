use crate::{
    codegen::utils::indent, models::Definition, ConstType, ConstValueDef, FieldDef, ModelDef,
    StringOrInteger, StructDef, Type, VariantDef,
};
use std::path::Path;
use std::{borrow::Cow, fmt::Write, path::PathBuf};

use super::context::Context;
use super::{spec_folder::Entry, utils::multiline_prefix_with};

#[derive(Default)]
pub struct RsSerde {}

impl super::Codegen for RsSerde {
    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        let context = Context::new_from_folder(folder)?;

        context.folder_tree().foreach_entry_recursively(|entry| {
            let outputs = render_folder(entry).unwrap();
            for (file_relative_path, content) in outputs {
                let file_path = output.join(file_relative_path);
                println!("write output to {:?}", file_path);
                std::fs::write(file_path, content).unwrap();
            }
        });

        for (spec, _) in context.iter_specs() {
            let output = {
                let mut output = output.clone();
                output.push(spec);
                output.set_extension("rs");
                output
            };

            println!("generating spec={spec:?} output={output:?}");

            let parent_folder = output.parent().unwrap();
            std::fs::create_dir_all(parent_folder)?;

            let code = render(&spec, &context)?;

            std::fs::write(&output, code).unwrap();
            println!("write output to {:?}", output);
        }

        Ok(())
    }
}

fn render_folder(entry: &Entry) -> anyhow::Result<Vec<(PathBuf, String)>> {
    if entry.is_empty() {
        // this is a leaf node, continue
        return Ok(vec![]);
    }

    let mut outputs = vec![];
    let children = entry.iter_child().collect::<Vec<_>>();

    let mut code = "".to_string();
    for child in children {
        writeln!(
            code,
            "pub mod {};",
            child.path().file_stem().unwrap().to_str().unwrap()
        )
        .unwrap();
    }

    outputs.push((entry.path().join("mod.rs"), code));

    Ok(outputs)
}

fn render(spec_path: &Path, context: &Context) -> anyhow::Result<String> {
    let def = context.get_definition(spec_path)?;
    let def = &def;
    let mut result = String::new();

    for include in def.includes.iter() {
        let include_path = context.get_include_path(&include.namespace, spec_path)?;
        let relative_path = pathdiff::diff_paths(&include_path, spec_path).unwrap();

        let include_name = relative_path.file_stem().unwrap().to_str().unwrap();

        let mut mod_path = "".to_string();

        let relative_path_components = relative_path.components().collect::<Vec<_>>();
        for (idx, component) in relative_path_components.iter().enumerate() {
            match component {
                std::path::Component::ParentDir => {
                    mod_path.push_str("super::");
                }
                std::path::Component::CurDir => {
                    // do nothing
                }
                std::path::Component::Normal(name) => {
                    if idx + 1 < relative_path_components.len() {
                        mod_path.push_str(&format!("{}::", name.to_str().unwrap()));
                    } else {
                        break;
                    }
                }
                std::path::Component::Prefix(_) | std::path::Component::RootDir => unimplemented!(),
            }
        }
        mod_path.push_str(include_name);

        if let Some(rs_mod) = include.attributes.get("rs_mod") {
            // rs-mod overwrite
            mod_path = rs_mod.to_string()
        }

        if mod_path.eq(&include.namespace) {
            writeln!(result, "use {};", mod_path)?;
        } else {
            writeln!(result, "use {} as {};", mod_path, include.namespace)?;
        }
    }

    if !def.includes.is_empty() {
        writeln!(result, "")?;
    }

    let mut model_codes = vec![];

    for model in def.models.iter() {
        model_codes.push("".to_string());
        let model_code = model_codes.last_mut().unwrap();

        let model_name = &model.name;

        writeln!(model_code, "")?;
        writeln!(
            model_code,
            "{}",
            multiline_prefix_with(
                model
                    .desc
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or(model_name.as_str()),
                "/// "
            )
        )?;

        let mut derived = vec!["Debug", "serde::Serialize", "serde::Deserialize"];

        if let Some(extra_derived) = model.attribute("rs_extra_derive") {
            derived.extend(extra_derived.split(",").map(|d| d.trim()));
        }

        match &model.type_ {
            crate::ModelType::Enum { variants } => {
                let code = render_enum(model, &derived, variants, def)?;
                writeln!(model_code, "{}", code.trim())?;
            }
            crate::ModelType::Struct(struct_def) => {
                let code = render_struct(&model_name, &derived, struct_def, &def)?;
                writeln!(model_code, "{}", code.trim())?;
            }

            crate::ModelType::Virtual(struct_def) => {
                writeln!(model_code, "pub trait {} {{", &model.name)?;
                for field in struct_def.fields.iter() {
                    let field_name = &field.name;
                    let (field_name_rs, _) = to_identifier(field_name);

                    if let Some(desc) = &field.desc {
                        let comment = indent(multiline_prefix_with(desc, "/// "), 1);
                        writeln!(model_code, "{comment}",)?;
                    }

                    let field_type = field.rs_type();

                    writeln!(
                        model_code,
                        "    fn {field_name_rs}(&self) -> &{field_type};",
                    )?;

                    writeln!(
                        model_code,
                        "    fn set_{field_name_rs}(&mut self, value: {field_type}) -> {field_type};",
                    )?;
                }
                writeln!(model_code, "}}")?;
            }

            crate::ModelType::NewType { inner_type } => {
                let code = render_new_type(model_name, &derived, inner_type)?;
                writeln!(model_code, "{}", code.trim())?;
            }
            crate::ModelType::Const { value_type, values } => {
                let code = render_const(&model_name, &derived, value_type, &values)?;
                writeln!(model_code, "{}", code.trim())?;
            }
        }

        *model_code = super::utils::format_rust_code(model_code.as_str())?;
    }

    for (idx, model_code) in model_codes.into_iter().enumerate() {
        // prepend a new line
        if idx != 0 {
            writeln!(result, "")?;
        }
        writeln!(result, "{}", model_code.trim())?;
    }

    Ok(result)
}

fn render_derived(derived: &[&str]) -> String {
    format!(
        "#[derive({})]",
        derived
            .iter()
            .map(|d| format!("{},", d))
            .collect::<Vec<_>>()
            .join("")
    )
}

fn render_struct(
    model_name: &str,
    derived: &[&str],
    struct_def: &StructDef,
    def: &Definition,
) -> anyhow::Result<String> {
    let mut result = "".to_string();
    let model_code = &mut result;

    {
        writeln!(model_code, "{}", render_derived(&derived))?;
        writeln!(model_code, "pub struct {model_name} {{")?;

        let mut fields = vec![];
        if let Some(virtual_name) = &struct_def.extend {
            match def.get_model(&virtual_name) {
                Some(model) => match &model.type_ {
                    crate::ModelType::Virtual(struct_def) => {
                        fields.extend(struct_def.fields.clone());
                    }
                    _ => {
                        anyhow::bail!("model is not virtual: {}", virtual_name);
                    }
                },
                None => anyhow::bail!("not able to find virtual model: {}", virtual_name),
            }
        }

        fields.extend(struct_def.fields.clone());

        let fields_def_code = render_fields_def(&fields)?;
        writeln!(model_code, "{}", indent(fields_def_code, 1))?;

        writeln!(model_code, "}}")?;
    }

    if let Some(virtual_name) = &struct_def.extend {
        writeln!(model_code, "")?;
        writeln!(model_code, "impl {virtual_name} for {model_name} {{")?;
        match def.get_model(&virtual_name) {
            Some(model) => match &model.type_ {
                crate::ModelType::Virtual(struct_def) => {
                    for field in struct_def.fields.iter() {
                        let field_name = &field.name;
                        let (field_name_rs, _) = to_identifier(field_name);
                        let field_type = field.rs_type();
                        writeln!(
                            model_code,
                            "    fn {field_name_rs}(&self) -> &{field_type} {{",
                        )?;
                        writeln!(model_code, "        &self.{field_name_rs}")?;
                        writeln!(model_code, "    }}",)?;

                        writeln!(
                            model_code,
                            "    fn set_{field_name_rs}(&mut self, value: {field_type}) -> {field_type} {{",
                        )?;
                        writeln!(
                            model_code,
                            "        std::mem::replace(&mut self.{field_name_rs},  value)"
                        )?;
                        writeln!(model_code, "    }}",)?;
                    }
                }
                _ => {
                    anyhow::bail!("model is not virtual: {}", virtual_name);
                }
            },
            None => anyhow::bail!("not able to find virtual model: {}", virtual_name),
        }
        writeln!(model_code, "}}")?;
    }

    Ok(result)
}

fn render_fields_def(fields: &[FieldDef]) -> anyhow::Result<String> {
    let mut result = "".to_string();
    let code = &mut result;
    for field in fields.iter() {
        if let Some(desc) = &field.desc {
            let comment = multiline_prefix_with(desc, "/// ");
            writeln!(code, "{}", comment)?;
        }

        for attr in field.rs_attributes() {
            writeln!(code, "#[{attr}]")?;
        }

        let field_name = &field.name;
        let (field_name_rs, modified) = to_identifier(field_name);

        if modified {
            writeln!(code, "#[serde(rename = \"{field_name}\")]")?;
        }
        writeln!(code, "pub {}: {},", field_name_rs, field.rs_type())?;
    }
    Ok(result)
}

fn render_enum(
    model: &ModelDef,
    derived: &[&str],
    variants: &[VariantDef],
    def: &Definition,
) -> anyhow::Result<String> {
    let model_name = &model.name;

    let mut result = "".to_string();
    let model_code = &mut result;
    match model.attribute("rs_enum_variant_type").map(String::as_str) {
        Some("true") => {
            // create separate type for each variant
            writeln!(model_code, "{}", render_derived(&derived))?;
            writeln!(
                model_code,
                "#[serde(tag = \"type\", content = \"payload\")]"
            )?;
            writeln!(model_code, "pub enum {} {{", &model.name)?;

            for variant in variants {
                let variant_name = &variant.name;
                let variant_type_name = format!("{model_name}{variant_name}");

                if let Some(desc) = &variant.desc {
                    let comment = multiline_prefix_with(desc, "/// ");
                    writeln!(model_code, "{}", indent(&comment, 1))?;
                }
                writeln!(model_code, "    {variant_name}({variant_type_name}),",)?;
            }
            writeln!(model_code, "}}")?;

            for variant in variants {
                let variant_name = &variant.name;
                let variant_type_name = format!("{model_name}{variant_name}");

                let mut code = "".to_string();
                let code = &mut code;

                if let Some(payload_type) = &variant.payload_type {
                    let payload_type = payload_type.rs_type();
                    writeln!(code, "{}", render_derived(&derived))?;
                    writeln!(code, "pub struct {variant_type_name}({payload_type});")?;
                } else if let Some(fields) = &variant.payload_fields {
                    let struct_def = StructDef {
                        extend: None,
                        fields: fields.clone(),
                    };
                    let struct_code =
                        render_struct(&variant_type_name, &derived, &struct_def, def)?;
                    writeln!(code, "{}", struct_code)?;
                } else {
                    writeln!(code, "{}", render_derived(&derived))?;
                    writeln!(code, "pub struct {variant_type_name};")?;
                }

                writeln!(code, "impl Into<{model_name}> for {variant_type_name} {{")?;
                writeln!(code, "    fn into(self) -> {model_name} {{")?;
                writeln!(code, "        {model_name}::{variant_name}(self)")?;
                writeln!(code, "    }}")?;
                writeln!(code, "}}")?;

                writeln!(model_code, "{}", code)?;
            }
        }
        _ => {
            // create separate type for each variant
            writeln!(model_code, "{}", render_derived(&derived))?;
            writeln!(
                model_code,
                "#[serde(tag = \"type\", content = \"payload\")]"
            )?;
            writeln!(model_code, "pub enum {} {{", &model.name)?;

            for variant in variants {
                if let Some(desc) = &variant.desc {
                    let comment = multiline_prefix_with(desc, "/// ");
                    writeln!(model_code, "{}", indent(&comment, 1))?;
                }

                if let Some(payload_type) = &variant.payload_type {
                    writeln!(
                        model_code,
                        "    {}({}),",
                        variant.name,
                        payload_type.rs_type()
                    )?;
                } else if let Some(fields) = &variant.payload_fields {
                    let fields_def_code = render_fields_def(&fields)?;

                    writeln!(model_code, "    {} {{", variant.name,)?;
                    writeln!(model_code, "{}", indent(&fields_def_code, 2))?;
                    writeln!(model_code, "    }},")?;
                } else {
                    writeln!(model_code, "    {},", variant.name,)?;
                }
            }

            writeln!(model_code, "}}")?;
        }
    }
    Ok(result)
}

fn render_new_type(
    model_name: &str,
    derived: &[&str],
    inner_type: &Type,
) -> anyhow::Result<String> {
    let mut result = "".to_string();
    writeln!(result, "{}", render_derived(derived))?;
    writeln!(
        result,
        "pub struct {model_name}(pub {});",
        inner_type.rs_type()
    )?;
    Ok(result)
}

fn extend_derived<'a>(derived: &[&'a str], more: &[&'a str]) -> Vec<&'a str> {
    let mut derived = derived.to_vec();

    for d in more.iter() {
        if !derived.contains(d) {
            derived.push(d);
        }
    }

    derived
}

fn render_const(
    model_name: &str,
    derived: &[&str],
    value_type: &ConstType,
    values: &[ConstValueDef],
) -> anyhow::Result<String> {
    let mut code = "".to_string();
    let value_type_in_struct = value_type.rs_type();
    let value_type_in_to_value = value_type_in_struct;
    let value_type_in_from_value = match value_type {
        ConstType::I8 => "i8",
        ConstType::I16 => "i16",
        ConstType::I32 => "i32",
        ConstType::I64 => "i64",
        // from_value able to accept &str for all lifetime
        ConstType::String => "&str",
    };

    // for const, we should always derive, "Copy", "Clone", "Hash", "Ord" like
    let derived = extend_derived(
        derived,
        &[
            "Copy",
            "Clone",
            "Hash",
            "PartialEq",
            "Eq",
            "PartialOrd",
            "Ord",
        ],
    );

    writeln!(code, "{}", render_derived(&derived))?;
    writeln!(code, "pub struct {model_name}(pub {value_type_in_struct});")?;

    {
        // generate from_value and to_value
        writeln!(code, "")?;
        writeln!(code, "impl {model_name} {{")?;

        let from_value = {
            // from_value
            let mut code = "".to_string();
            writeln!(
                code,
                "pub fn from_value(val: {value_type_in_from_value}) -> Option<Self> {{"
            )?;
            writeln!(code, "    match val {{")?;
            for value in values.iter() {
                let value_name = rs_const_name(&value.name);
                let value_literal = rs_const_literal(&value.value);
                writeln!(code, "        {value_literal} => Some(Self::{value_name}),")?;
            }
            writeln!(code, "        _ => None,")?;

            writeln!(code, "    }}")?;
            writeln!(code, "}}")?;
            code
        };

        writeln!(code, "{}", indent(&from_value.trim(), 1))?;

        let to_value = {
            // from_value
            let mut code = "".to_string();
            writeln!(code, "pub fn to_value(self) -> {value_type_in_to_value} {{")?;
            writeln!(code, "    self.0")?;
            writeln!(code, "}}")?;
            code
        };

        writeln!(code, "{}", indent(&to_value.trim(), 1))?;

        writeln!(code, "}}")?;
    }

    writeln!(code, "")?;
    writeln!(code, "impl {model_name} {{")?;

    for value in values.iter() {
        let value_name = rs_const_name(&value.name);
        let value_literal = rs_const_literal(&value.value);
        if let Some(desc) = &value.desc {
            let comment = indent(multiline_prefix_with(desc, "/// "), 1);
            writeln!(code, "{comment}")?;
        }

        writeln!(
            code,
            "    pub const {value_name}: {model_name} = {model_name}({value_literal});"
        )?;
    }

    writeln!(code, "}}")?;
    Ok(code)
}

fn rs_const_name(name: &str) -> String {
    use convert_case::{Case, Casing};
    name.to_case(Case::UpperSnake)
}

fn rs_const_literal(val: &StringOrInteger) -> String {
    match val {
        StringOrInteger::String(s) => format!("\"{s}\""),
        StringOrInteger::Integer(i) => i.to_string(),
    }
}

fn to_identifier(name: &str) -> (Cow<str>, bool) {
    match name {
        "as" | "use" | "extern crate" | "break" | "const" | "continue" | "crate" | "else"
        | "if" | "if let" | "enum" | "extern" | "false" | "fn" | "for" | "impl" | "in" | "let"
        | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" | "Self"
        | "self" | "static" | "struct" | "super" | "trait" | "true" | "type" | "unsafe"
        | "where" | "while" | "abstract" | "alignof" | "become" | "box" | "do" | "final"
        | "macro" | "offsetof" | "override" | "priv" | "proc" | "pure" | "sizeof" | "typeof"
        | "unsized" | "virtual" | "yield" => (format!("{name}_").into(), true),
        _ => (name.into(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        fn test_def(spec: &Path, code_path: &str) {
            println!("{spec:?}");
            let context = Context::new();
            let rendered = super::render(spec, &context).unwrap();
            let rendered_ast = syn::parse_file(&mut rendered.clone()).unwrap();

            let code = std::fs::read_to_string(code_path).unwrap();
            let code_ast = syn::parse_file(&mut code.to_string()).unwrap();

            let rendered_pretty = prettyplease::unparse(&rendered_ast);
            let code_pretty = prettyplease::unparse(&code_ast);

            #[cfg(not(feature = "test_update_spec"))]
            pretty_assertions::assert_eq!(code_pretty.trim(), rendered_pretty.as_str().trim());

            #[cfg(feature = "test_update_spec")]
            {
                if rendered_pretty.trim() != code_pretty.as_str().trim() {
                    std::fs::write(code_path, rendered_pretty).unwrap();
                }
            }
        }

        for (spec, expected) in &[
            (
                "src/codegen/fixtures/specs/simple_struct.yaml",
                "src/codegen/fixtures/rs_serde/simple_struct.rs",
            ),
            (
                "src/codegen/fixtures/specs/extend.yaml",
                "src/codegen/fixtures/rs_serde/extend.rs",
            ),
            (
                "src/codegen/fixtures/specs/const_i8.yaml",
                "src/codegen/fixtures/rs_serde/const_i8.rs",
            ),
            (
                "src/codegen/fixtures/specs/const_i16.yaml",
                "src/codegen/fixtures/rs_serde/const_i16.rs",
            ),
            (
                "src/codegen/fixtures/specs/const_i64.yaml",
                "src/codegen/fixtures/rs_serde/const_i64.rs",
            ),
            (
                "src/codegen/fixtures/specs/const_string.yaml",
                "src/codegen/fixtures/rs_serde/const_string.rs",
            ),
            (
                "src/codegen/fixtures/specs/include_test.yaml",
                "src/codegen/fixtures/rs_serde/include_test.rs",
            ),
            (
                "src/codegen/fixtures/specs/enum.yaml",
                "src/codegen/fixtures/rs_serde/enum.rs",
            ),
            (
                "src/codegen/fixtures/specs/enum_variant_type.yaml",
                "src/codegen/fixtures/rs_serde/enum_variant_type.rs",
            ),
            (
                "src/codegen/fixtures/specs/enum_variant_fields.yaml",
                "src/codegen/fixtures/rs_serde/enum_variant_fields.rs",
            ),
            (
                "src/codegen/fixtures/specs/new_type.yaml",
                "src/codegen/fixtures/rs_serde/new_type.rs",
            ),
            (
                "src/codegen/fixtures/specs/json.yaml",
                "src/codegen/fixtures/rs_serde/json.rs",
            ),
            (
                "src/codegen/fixtures/specs/decimal.yaml",
                "src/codegen/fixtures/rs_serde/decimal.rs",
            ),
            (
                "src/codegen/fixtures/specs/bigint.yaml",
                "src/codegen/fixtures/rs_serde/bigint.rs",
            ),
            (
                "src/codegen/fixtures/specs/rs_keyword.yaml",
                "src/codegen/fixtures/rs_serde/keyword.rs",
            ),
        ] {
            test_def(PathBuf::from(spec).as_path(), &expected);
        }
    }
}
