use crate::models::Definition;
use std::fmt::Write;

pub fn render(def: &Definition) -> anyhow::Result<String> {
    let mut result = String::new();

    for model in def.models.iter() {
        writeln!(
            &mut result,
            "\n/// {}",
            model
                .desc
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(model.name.as_str())
        )?;

        let mut derived = vec!["Debug", "serde::Serialize", "serde::Deserialize"];

        if let Some(extra_derived) = model.attribute("rs_extra_derive") {
            derived.extend(extra_derived.split(",").map(|d| d.trim()));
        }

        match &model.type_ {
            crate::ModelType::Enum { variants } => {
                writeln!(&mut result, "#[derive({})]", derived.join(", "))?;
                writeln!(
                    &mut result,
                    "#[serde(tag = \"type\", content = \"payload\")]"
                )?;
                writeln!(&mut result, "pub enum {} {{", &model.name)?;

                for variant in variants {
                    if let Some(desc) = &variant.desc {
                        writeln!(&mut result, "    /// {desc}")?;
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
                writeln!(&mut result, "#[derive({})]", derived.join(", "))?;
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
                        writeln!(&mut result, "    /// {desc}")?;
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
                        writeln!(&mut result, "    /// {desc}",)?;
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
                writeln!(&mut result, "#[derive({})]", derived.join(", "))?;
                writeln!(
                    &mut result,
                    "pub struct {}(pub {});",
                    &model.name,
                    inner_type.rs_type()
                )?;
            }
        }
    }

    Ok(result)
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
                models,
                meta: Default::default(),
            };
            let rendered = super::render(&definition).unwrap();
            let rendered_ast = syn::parse_file(&mut rendered.clone()).unwrap();
            let code_ast = syn::parse_file(&mut code.to_string()).unwrap();

            let rendered_pretty = prettyplease::unparse(&rendered_ast);
            let code_pretty = prettyplease::unparse(&code_ast);

            if rendered_pretty.ne(&code_pretty) {
                println!("=== rendered:\n{}", rendered.as_str().trim());
                println!("=== expected:\n{}", code.trim());
                assert!(false, "code not match");
            }
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
                            payload_type: Type::I64.into(),
                            desc: Some("Variant I64".into()),
                        },
                        VariantDef {
                            name: "F64".into(),
                            payload_type: Type::F64.into(),
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
    }
}
