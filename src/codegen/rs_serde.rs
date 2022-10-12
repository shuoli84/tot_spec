use crate::models::Definition;
use std::fmt::Write;

pub fn render(def: &Definition) -> anyhow::Result<String> {
    let mut result = String::new();

    for model in def.models.iter() {
        writeln!(&mut result, "\n/// {}", model.name)?;

        match &model.type_ {
            crate::ModelType::Enum { variants } => {
                writeln!(
                    &mut result,
                    "#[derive(Debug, serde::Serialize, serde::Deserialize)]"
                )?;
                writeln!(&mut result, "pub enum {} {{", &model.name)?;

                for variant in variants {
                    writeln!(
                        &mut result,
                        "    {}({}),",
                        variant.name,
                        variant.playload_type.rs_type()
                    )?;
                }

                writeln!(&mut result, "}}")?;
            }
            crate::ModelType::Struct(struct_def) => {
                writeln!(
                    &mut result,
                    "#[derive(Debug, serde::Serialize, serde::Deserialize)]"
                )?;
                writeln!(&mut result, "pub struct {} {{", &model.name)?;

                if let Some(virtual_name) = &struct_def.extend {
                    match def.get_model(&virtual_name) {
                        Some(model) => match &model.type_ {
                            crate::ModelType::Virtual(struct_def) => {
                                for field in struct_def.fields.iter() {
                                    writeln!(
                                        &mut result,
                                        "    pub {}: {},",
                                        field.name,
                                        field.rs_type()
                                    )?;
                                }
                            }
                            _ => {
                                anyhow::bail!("model is not virtual: {}", virtual_name);
                            }
                        },
                        None => anyhow::bail!("not able to find virtual model: {}", virtual_name),
                    }
                }

                for field in struct_def.fields.iter() {
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
                writeln!(
                    &mut result,
                    "#[derive(Debug, serde::Serialize, serde::Deserialize)]"
                )?;
                writeln!(
                    &mut result,
                    "pub struct {}({});",
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
            let definition = Definition { models };
            let rendered = super::render(&definition).unwrap();
            if !rendered.as_str().trim().eq(code.trim()) {
                println!("=== rendered:\n{}", rendered.as_str().trim());
                println!("=== expected:\n{}", code.trim());
                assert!(false, "code not match");
            }
        }

        test_model_codegen(
            ModelDef {
                name: "SimpleStruct".to_string(),
                type_: ModelType::Struct(StructDef {
                    extend: None,
                    fields: vec![
                        FieldDef::new("bool_value", Type::Bool).with_required(true),
                        FieldDef::new("i8_value", Type::I8).with_required(true),
                        FieldDef::new("i64_value", Type::I64),
                        FieldDef::new("string_value", Type::String),
                        FieldDef::new("bytes_value", Type::Bytes),
                        FieldDef::new("i8_to_string", Type::map(Type::I8, Type::String))
                            .with_attribute(
                                "rs_type",
                                "std::collections::BTreeMap<i8, std::string::String>",
                            ),
                        FieldDef::new("key_values", Type::reference("KeyValue")),
                        FieldDef::new("children", Type::list(Type::reference("SimpleStruct"))),
                    ],
                }),
            },
            include_str!("fixtures/rs_serde/simple_struct.rs"),
        );
        test_model_codegen(
            ModelDef {
                name: "KeyValue".into(),
                type_: ModelType::new_type(Type::map(Type::String, Type::Bytes)),
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
                            playload_type: Type::I64,
                        },
                        VariantDef {
                            name: "F64".into(),
                            playload_type: Type::F64,
                        },
                    ],
                },
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
            include_str!("fixtures/rs_serde/extend.rs"),
        );
    }
}
