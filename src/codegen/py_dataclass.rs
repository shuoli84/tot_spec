use crate::{Definition, FieldDef, Type};
use std::fmt::Write;

use super::utils;

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

                let to_dict = generate_to_dict(&struct_def.fields)?;
                writeln!(&mut result, "{}", utils::indent(&to_dict, 1))?;
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

fn generate_to_dict(fields: &[FieldDef]) -> anyhow::Result<String> {
    let mut result = "".to_string();
    writeln!(&mut result, "def to_dict(self):")?;
    writeln!(&mut result, "    result = {{}}")?;

    for field in fields {
        match &field.type_ {
            Type::Unit => {
                // pass
            }
            Type::Bytes => {
                // todo, base64?
                writeln!(
                    &mut result,
                    "    result[\"{field_name}\"] = self.{field_name}",
                    field_name = field.name,
                )?;
            }
            Type::I64 | Type::I8 | Type::Bool | Type::F64 | Type::String => {
                writeln!(
                    &mut result,
                    "    result[\"{field_name}\"] = self.{field_name}",
                    field_name = field.name,
                )?;
            }

            Type::List { item_type } => {
                let tmp_var_name = format!("{}_tmp", field.name);
                let field_name = &field.name;

                writeln!(&mut result, "    {tmp_var_name} = []")?;
                writeln!(&mut result, "    for item in self.{field_name} or []:",)?;
                writeln!(&mut result, "        {tmp_var_name}.append(item.to_dict())")?;
                writeln!(&mut result, "    result[\"{field_name}\"] = {tmp_var_name}")?;
            }
            Type::Map {
                key_type,
                value_type,
            } => {
                let tmp_var_name = format!("{}_tmp", field.name);
                let field_name = &field.name;

                writeln!(&mut result, "    {tmp_var_name} = {{}}")?;
                writeln!(
                    &mut result,
                    "    for key, item in (self.{field_name} or {{}}).items():",
                )?;
                writeln!(&mut result, "        {tmp_var_name}[key] = item.to_dict()")?;
                writeln!(&mut result, "    result[\"{field_name}\"] = {tmp_var_name}")?;
            }
            Type::Reference { target } => {
                let field_name = &field.name;
                writeln!(
                    &mut result,
                    "    result[\"{field_name}\"] = self.{field_name}.to_dict()"
                )?;
            }
        }
    }

    writeln!(&mut result, "    return result")?;

    Ok(result)
}
