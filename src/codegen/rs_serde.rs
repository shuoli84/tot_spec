use crate::{
    codegen::utils::indent, models::Definition, ConstType, ConstValueDef, StringOrInteger, Type,
};
use std::fmt::Write;

use super::utils::multiline_prefix_with;

pub fn render(def: &Definition) -> anyhow::Result<String> {
    let mut result = String::new();

    for include in def.includes.iter() {
        let mod_path = include
            .attributes
            .get("rs_mod")
            .unwrap_or(&include.namespace);
        if mod_path.eq(&include.namespace) {
            writeln!(&mut result, "use {};", mod_path)?;
        } else {
            writeln!(&mut result, "use {} as {};", mod_path, include.namespace)?;
        }
    }

    for model in def.models.iter() {
        let model_name = &model.name;

        writeln!(&mut result, "")?;
        writeln!(
            &mut result,
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
                writeln!(&mut result, "{}", render_derived(&derived))?;
                writeln!(
                    &mut result,
                    "#[serde(tag = \"type\", content = \"payload\")]"
                )?;
                writeln!(&mut result, "pub enum {} {{", &model.name)?;

                for variant in variants {
                    if let Some(desc) = &variant.desc {
                        let comment = multiline_prefix_with(desc, "/// ");
                        writeln!(&mut result, "{}", indent(&comment, 1))?;
                    }

                    if let Some(payload_type) = &variant.payload_type {
                        writeln!(
                            &mut result,
                            "    {}({}),",
                            variant.name,
                            payload_type.rs_type()
                        )?;
                    } else {
                        writeln!(&mut result, "    {},", variant.name,)?;
                    }
                }

                writeln!(&mut result, "}}")?;
            }
            crate::ModelType::Struct(struct_def) => {
                writeln!(&mut result, "{}", render_derived(&derived))?;
                writeln!(&mut result, "pub struct {} {{", &model.name)?;

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

                for field in fields.iter() {
                    if let Some(desc) = &field.desc {
                        let comment = multiline_prefix_with(desc, "/// ");
                        writeln!(&mut result, "{}", indent(&comment, 1))?;
                    }

                    writeln!(&mut result, "    pub {}: {},", field.name, field.rs_type())?;
                }

                writeln!(&mut result, "}}")?;

                if let Some(virtual_name) = &struct_def.extend {
                    writeln!(&mut result, "\nimpl {} for {} {{", virtual_name, model.name)?;
                    match def.get_model(&virtual_name) {
                        Some(model) => match &model.type_ {
                            crate::ModelType::Virtual(struct_def) => {
                                for field in struct_def.fields.iter() {
                                    writeln!(
                                        &mut result,
                                        "    fn {}(&self) -> &{} {{",
                                        field.name,
                                        field.rs_type()
                                    )?;
                                    writeln!(&mut result, "        &self.{}", field.name)?;
                                    writeln!(&mut result, "    }}",)?;
                                }
                            }
                            _ => {
                                anyhow::bail!("model is not virtual: {}", virtual_name);
                            }
                        },
                        None => anyhow::bail!("not able to find virtual model: {}", virtual_name),
                    }
                    writeln!(&mut result, "}}")?;
                }
            }

            crate::ModelType::Virtual(struct_def) => {
                writeln!(&mut result, "pub trait {} {{", &model.name)?;
                for field in struct_def.fields.iter() {
                    if let Some(desc) = &field.desc {
                        let comment = indent(multiline_prefix_with(desc, "/// "), 1);
                        writeln!(&mut result, "{comment}",)?;
                    }

                    writeln!(
                        &mut result,
                        "    fn {}(&self) -> &{};",
                        field.name,
                        field.rs_type()
                    )?;
                }
                writeln!(&mut result, "}}")?;
            }

            crate::ModelType::NewType { inner_type } => {
                let code = render_new_type(model_name, &derived, inner_type)?;
                writeln!(&mut result, "{}", code.trim())?;
            }
            crate::ModelType::Const { value_type, values } => {
                let code = render_const(&model_name, &derived, value_type, &values)?;
                writeln!(&mut result, "{}", code.trim())?;
            }
        }
    }

    let ast = syn::parse_file(&mut result)?;
    let result = prettyplease::unparse(&ast);

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

fn render_new_type(
    model_name: &str,
    derived: &[&str],
    inner_type: &Type,
) -> anyhow::Result<String> {
    let mut result = "".to_string();
    writeln!(&mut result, "{}", render_derived(derived))?;
    writeln!(
        &mut result,
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

    writeln!(&mut code, "{}", render_derived(&derived))?;
    writeln!(
        &mut code,
        "pub struct {model_name}(pub {value_type_in_struct});"
    )?;

    {
        // generate from_value and to_value
        writeln!(&mut code, "")?;
        writeln!(&mut code, "impl {model_name} {{")?;

        let from_value = {
            // from_value
            let mut code = "".to_string();
            writeln!(
                &mut code,
                "pub fn from_value(val: {value_type_in_from_value}) -> Option<Self> {{"
            )?;
            writeln!(&mut code, "    match val {{")?;
            for value in values.iter() {
                let value_name = rs_const_name(&value.name);
                let value_literal = rs_const_literal(&value.value);
                writeln!(
                    &mut code,
                    "        {value_literal} => Some(Self::{value_name}),"
                )?;
            }
            writeln!(&mut code, "        _ => None,")?;

            writeln!(&mut code, "    }}")?;
            writeln!(&mut code, "}}")?;
            code
        };

        writeln!(&mut code, "{}", indent(&from_value.trim(), 1))?;

        let to_value = {
            // from_value
            let mut code = "".to_string();
            writeln!(
                &mut code,
                "pub fn to_value(self) -> {value_type_in_to_value} {{"
            )?;
            writeln!(&mut code, "    self.0")?;
            writeln!(&mut code, "}}")?;
            code
        };

        writeln!(&mut code, "{}", indent(&to_value.trim(), 1))?;

        writeln!(&mut code, "}}")?;
    }

    writeln!(&mut code, "")?;
    writeln!(&mut code, "impl {model_name} {{")?;

    for value in values.iter() {
        let value_name = rs_const_name(&value.name);
        let value_literal = rs_const_literal(&value.value);
        if let Some(desc) = &value.desc {
            let comment = indent(multiline_prefix_with(desc, "/// "), 1);
            writeln!(&mut code, "{comment}")?;
        }

        writeln!(
            &mut code,
            "    pub const {value_name}: {model_name} = {model_name}({value_literal});"
        )?;
    }

    writeln!(&mut code, "}}")?;
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

#[cfg(test)]
mod tests {
    use crate::models::*;

    #[test]
    fn test_render() {
        fn test_model_codegen(model: ModelDef, code: &str) {
            test_models_codegen(vec![model], code)
        }

        fn test_models_codegen(models: Vec<ModelDef>, code: &str) {
            let definition = Definition {
                includes: vec![],
                models,
                meta: Default::default(),
            };
            test_def(definition, code)
        }

        fn test_def(definition: Definition, code: &str) {
            let rendered = super::render(&definition).unwrap();
            let rendered_ast = syn::parse_file(&mut rendered.clone()).unwrap();
            let code_ast = syn::parse_file(&mut code.to_string()).unwrap();

            let rendered_pretty = prettyplease::unparse(&rendered_ast);
            let code_pretty = prettyplease::unparse(&code_ast);

            pretty_assertions::assert_eq!(rendered_pretty, code_pretty);
        }

        test_model_codegen(
            ModelDef {
                name: "SimpleStruct".to_string(),
                desc: Some("A test struct with different kinds of fields".into()),
                type_: ModelType::Struct(StructDef {
                    extend: None,
                    fields: vec![
                        FieldDef::new("bool_value", Type::Bool)
                            .with_required(true)
                            .with_desc("required bool field"),
                        FieldDef::new("i8_value", Type::I8)
                            .with_required(true)
                            .with_desc("required i8 field"),
                        FieldDef::new("i64_value", Type::I64),
                        FieldDef::new("string_value", Type::String),
                        FieldDef::new("bytes_value", Type::Bytes),
                        FieldDef::new("string_map", Type::map(Type::String))
                            .with_desc("string map with customized Map type")
                            .with_attribute(
                                "rs_type",
                                "std::collections::BTreeMap<std::string::String, std::string::String>",
                            ),
                        FieldDef::new("key_values", Type::reference("KeyValue")),
                        FieldDef::new("children", Type::list(Type::reference("SimpleStruct"))),
                    ],
                }),
                ..Default::default()
            },
            include_str!("fixtures/rs_serde/simple_struct.rs"),
        );
        test_model_codegen(
            ModelDef {
                name: "KeyValue".into(),
                type_: ModelType::new_type(Type::map(Type::Bytes)),
                ..Default::default()
            },
            include_str!("fixtures/rs_serde/key_value.rs"),
        );

        test_model_codegen(
            ModelDef {
                name: "Number".into(),
                type_: ModelType::Enum {
                    variants: vec![
                        VariantDef {
                            name: "I64".into(),
                            payload_type: Some(Type::I64.into()),
                            desc: Some("Variant I64".into()),
                        },
                        VariantDef {
                            name: "F64".into(),
                            payload_type: Some(Type::F64.into()),
                            desc: Some("Variant F64".into()),
                        },
                    ],
                },
                ..ModelDef::default()
            },
            include_str!("fixtures/rs_serde/enum.rs"),
        );

        test_models_codegen(
            vec![
                ModelDef {
                    name: "Base".into(),
                    type_: ModelType::Virtual(StructDef {
                        extend: None,
                        fields: vec![FieldDef::new("request_id", Type::String)],
                    }),
                    ..ModelDef::default()
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
                    ..ModelDef::default()
                },
                ModelDef {
                    name: "ResetRequest".into(),
                    type_: ModelType::Struct(StructDef {
                        extend: Some("Base".into()),
                        fields: vec![],
                    }),
                    ..ModelDef::default()
                },
            ],
            include_str!("fixtures/rs_serde/extend.rs"),
        );

        test_model_codegen(
            ModelDef {
                name: "Id".into(),
                type_: ModelType::new_type(Type::reference("i64")),
                desc: Some("NewType to i64, and derive Ord macros".into()),
                ..ModelDef::default()
            }
            .with_attribute("rs_extra_derive", "PartialEq, Eq, PartialOrd, Ord"),
            include_str!("fixtures/rs_serde/new_type.rs"),
        );

        for (spec, expected) in &[
            (
                include_str!("fixtures/specs/const_i8.yaml"),
                include_str!("fixtures/rs_serde/const_i8.rs"),
            ),
            (
                include_str!("fixtures/specs/const_i64.yaml"),
                include_str!("fixtures/rs_serde/const_i64.rs"),
            ),
            (
                include_str!("fixtures/specs/const_string.yaml"),
                include_str!("fixtures/rs_serde/const_string.rs"),
            ),
            (
                include_str!("fixtures/specs/include_test.yaml"),
                include_str!("fixtures/rs_serde/include_test.rs"),
            ),
        ] {
            let def = serde_yaml::from_str::<Definition>(&spec).unwrap();
            test_def(def, expected);
        }
    }
}
