use crate::{Definition, FieldDef, Type};
use std::fmt::Write;

pub fn render(def: &Definition) -> anyhow::Result<String> {
    let mut result = String::new();

    writeln!(&mut result, "from dataclasses import dataclass")?;
    writeln!(&mut result, "import typing")?;
    writeln!(&mut result, "")?;

    for model in def.models.iter() {
        writeln!(&mut result, "\n# {}", model.name)?;

        match &model.type_ {
            // python has no built in enum, so we generate base class
            // and each variants as a separate class
            crate::ModelType::Enum { variants } => {
                writeln!(&mut result, "@dataclass")?;
                writeln!(&mut result, "class {}:", &model.name)?;
                writeln!(&mut result, "    pass")?;

                for variant in variants {
                    writeln!(
                        &mut result,
                        "\n# variant {} for {}",
                        variant.name, model.name
                    )?;
                    writeln!(&mut result, "@dataclass")?;
                    writeln!(
                        &mut result,
                        "class {enum_name}_{variant_name}({enum_name}):",
                        enum_name = model.name,
                        variant_name = variant.name
                    )?;
                    writeln!(
                        &mut result,
                        "    payload: {}",
                        py_type(&variant.playload_type),
                    )?;
                }
            }
            crate::ModelType::Struct(struct_def) | crate::ModelType::Virtual(struct_def) => {
                writeln!(&mut result, "@dataclass")?;

                if let Some(virtual_name) = &struct_def.extend {
                    writeln!(&mut result, "class {}({}):", model.name, virtual_name)?;
                } else {
                    writeln!(&mut result, "class {}:", model.name)?;
                };

                if struct_def.fields.is_empty() {
                    writeln!(&mut result, "    pass")?;
                } else {
                    for field in struct_def.fields.iter() {
                        if field.required {
                            writeln!(
                                &mut result,
                                "    {}: {}",
                                field.name,
                                py_type_for_field(&field)
                            )?;
                        } else {
                            // for optional field, use None as default value
                            writeln!(
                                &mut result,
                                "    {}: {} = None",
                                field.name,
                                py_type_for_field(&field)
                            )?;
                        }
                    }
                }
            }

            crate::ModelType::NewType { inner_type } => {
                writeln!(
                    &mut result,
                    "{} = typing.Type[{}]",
                    model.name,
                    py_type(inner_type)
                )?;
            }
        }
    }

    Ok(result)
}

fn py_type_for_field(field: &FieldDef) -> String {
    let field_type = py_type(&field.type_);

    if field.required {
        field_type
    } else {
        format!("typing.Optional[{}]", field_type)
    }
}

fn py_type(ty: &Type) -> String {
    match ty {
        Type::Unit => "None".into(),
        Type::Bool => "bool".into(),
        Type::I8 | Type::I64 => "int".into(),
        Type::F64 => "float".into(),
        Type::Bytes => "bytes".into(),
        Type::String => "str".into(),
        Type::List { item_type } => {
            format!("typing.List[{}]", py_type(item_type))
        }
        Type::Map {
            key_type,
            value_type,
        } => format!(
            "typing.Dict[{}, {}]",
            py_type(key_type),
            py_type(value_type)
        ),
        Type::Reference { target } => format!("'{}'", target),
    }
}
