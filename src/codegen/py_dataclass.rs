use crate::{Definition, FieldDef, Type};
use std::fmt::Write;

use super::utils::{self, indent};

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

                writeln!(&mut result, "")?;
                let to_dict = generate_to_dict(&struct_def.fields, &def)?;
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

fn generate_to_dict(fields: &[FieldDef], def: &Definition) -> anyhow::Result<String> {
    let mut result = "".to_string();
    writeln!(&mut result, "def to_dict(self):")?;
    writeln!(&mut result, "    result = {{}}")?;

    for field in fields {
        writeln!(&mut result, "\n    # {}", field.name)?;

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
                let field_to_dict = to_dict_for_one_field(item_type, "item", "item_tmp", &def)?;
                writeln!(&mut result, "{}", indent(&field_to_dict, 2))?;
                writeln!(&mut result, "        {tmp_var_name}.append(item_tmp)")?;
                writeln!(&mut result, "    result[\"{field_name}\"] = {tmp_var_name}")?;
            }
            Type::Map {
                key_type: _,
                value_type,
            } => {
                let tmp_var_name = format!("{}_tmp", field.name);
                let field_name = &field.name;

                writeln!(&mut result, "    {tmp_var_name} = {{}}")?;
                writeln!(
                    &mut result,
                    "    for key, item in (self.{field_name} or {{}}).items():",
                )?;
                let field_to_dict = to_dict_for_one_field(value_type, "item", "item_tmp", &def)?;
                writeln!(&mut result, "{}", indent(&field_to_dict, 2))?;
                writeln!(&mut result, "        {tmp_var_name}[key] = item_tmp")?;
                writeln!(&mut result, "    result[\"{field_name}\"] = {tmp_var_name}")?;
            }
            ty @ Type::Reference { .. } => {
                let field_name = &field.name;
                let tmp_var_name = format!("{}_tmp", field_name);
                writeln!(&mut result, "    {tmp_var_name} = {{}}")?;
                let to_dict = to_dict_for_one_field(
                    &ty,
                    &format!("self.{}", field_name),
                    &tmp_var_name,
                    def,
                )?;
                writeln!(&mut result, "{to_dict}", to_dict = indent(&to_dict, 1))?;
                writeln!(&mut result, "    result[\"{field_name}\"] = {tmp_var_name}")?;
            }
        }
    }

    writeln!(&mut result, "    return result")?;

    Ok(result)
}

fn to_dict_for_one_field(
    ty: &Type,
    in_var: &str,
    out_var: &str,
    def: &Definition,
) -> anyhow::Result<String> {
    Ok(match ty {
        Type::Unit | Type::Bool | Type::I8 | Type::I64 | Type::F64 | Type::Bytes | Type::String => {
            format!("{out_var} = {in_var}")
        }
        Type::List { item_type } => {
            let mut result = "".to_string();
            writeln!(&mut result, "for item in {in_var}:",)?;
            let field_to_dict = to_dict_for_one_field(item_type, "item", "item_tmp", &def)?;
            writeln!(&mut result, "{}", indent(&field_to_dict, 1))?;
            writeln!(&mut result, "    {out_var}.append(item_tmp)")?;
            result
        }
        Type::Map {
            key_type: _,
            value_type,
        } => {
            let mut result = "".to_string();
            writeln!(&mut result, "for key, item in {in_var}.items():")?;
            let field_to_dict = to_dict_for_one_field(value_type, "item", "item_tmp", &def)?;
            writeln!(&mut result, "{}", indent(&field_to_dict, 1))?;
            writeln!(&mut result, "    {out_var}[key] = item_tmp")?;
            result
        }
        Type::Reference { target } => {
            let target_model = def.get_model(target).unwrap();
            match &target_model.type_ {
                crate::ModelType::NewType { inner_type } => {
                    to_dict_for_one_field(&inner_type, in_var, out_var, def)?
                }
                _ => {
                    format!("{out_var} = {in_var}.to_dict()")
                }
            }
        }
    })
}
