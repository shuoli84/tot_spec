use crate::{Definition, FieldDef, Type};
use std::fmt::Write;

use super::utils::{self, indent};

pub fn render(def: &Definition) -> anyhow::Result<String> {
    let type_var_name = "type_";

    let mut result = String::new();

    writeln!(&mut result, "from dataclasses import dataclass")?;
    writeln!(&mut result, "import abc")?;
    writeln!(&mut result, "import typing")?;

    writeln!(&mut result, "")?;

    for model in def.models.iter() {
        let model_name = &model.name;
        let comment = if let Some(desc) = &model.desc {
            desc
        } else {
            model_name
        };

        writeln!(&mut result, "\n# {comment}")?;

        match &model.type_ {
            // python has no built in enum, so we generate base class
            // and each variants as a separate class
            crate::ModelType::Enum { variants } => {
                let enum_name = &model.name;
                writeln!(&mut result, "class {enum_name}(abc.ABC):")?;
                writeln!(&mut result, "    pass")?;

                // to_dict
                {
                    writeln!(&mut result, "")?;
                    writeln!(&mut result, "    @abc.abstractmethod")?;
                    writeln!(&mut result, "    def to_dict(self):")?;
                    writeln!(&mut result, "        pass")?;
                }

                // from_dict is the real impl, no variant sub class should provide
                // the impl
                let code_block = {
                    let mut code_block = "".to_string();
                    writeln!(&mut code_block, "")?;
                    writeln!(&mut code_block, "@staticmethod")?;
                    writeln!(&mut code_block, "def from_dict(d):")?;
                    writeln!(&mut code_block, "    {type_var_name} = d[\"type\"]")?;

                    for (variant_idx, variant) in variants.iter().enumerate() {
                        let variant_name = &variant.name;
                        let type_tag = variant_name.clone();
                        let variant_cls_name = format!("{enum_name}_{variant_name}");

                        if variant_idx == 0 {
                            writeln!(&mut code_block, "    if {type_var_name} == \"{type_tag}\":")?;
                        } else {
                            writeln!(
                                &mut code_block,
                                "    elif {type_var_name} == \"{type_tag}\":"
                            )?;
                        }

                        if let Some(payload_type) = &variant.payload_type {
                            writeln!(&mut code_block, "        payload = d[\"payload\"]")?;

                            let payload_from_dict = from_dict_for_one_field(
                                payload_type,
                                "payload",
                                "payload_tmp",
                                def,
                            )?;
                            writeln!(&mut code_block, "{}", indent(&payload_from_dict, 2))?;
                            writeln!(
                                &mut code_block,
                                "        return {variant_cls_name}(payload=payload_tmp)"
                            )?;
                        } else {
                            writeln!(&mut code_block, "        {variant_cls_name}()")?;
                        }
                    }

                    writeln!(&mut code_block, "    else:")?;
                    writeln!(
                        &mut code_block,
                        "        raise ValueError(f\"invalid type: {{{type_var_name}}}\")"
                    )?;

                    code_block
                };
                writeln!(&mut result, "{}", indent(&code_block, 1))?;

                // generate sub class for each variant
                for variant in variants {
                    let variant_name = &variant.name;

                    let mut variant_code = "".to_string();
                    writeln!(
                        &mut variant_code,
                        "\n# variant {} for {}",
                        variant.name, model.name
                    )?;
                    writeln!(&mut variant_code, "@dataclass")?;
                    writeln!(
                        &mut variant_code,
                        "class {enum_name}_{variant_name}({enum_name}):",
                        enum_name = model.name,
                        variant_name = variant.name
                    )?;
                    if let Some(payload_type) = &variant.payload_type {
                        writeln!(&mut variant_code, "    payload: {}", py_type(&payload_type))?;
                    } else {
                        writeln!(&mut variant_code, "    pass")?;
                    }

                    // to_dict
                    {
                        writeln!(&mut variant_code, "")?;
                        writeln!(&mut variant_code, "    def to_dict(self):")?;
                        writeln!(
                            &mut variant_code,
                            "        {type_var_name} = \"{variant_name}\""
                        )?;

                        if let Some(payload_type) = &variant.payload_type {
                            let payload_to_dict = to_dict_for_one_field(
                                &payload_type,
                                "self.payload",
                                "payload_tmp",
                                def,
                            )?;
                            writeln!(&mut variant_code, "{}", indent(&payload_to_dict, 2))?;
                            writeln!(&mut variant_code, "        return {{")?;
                            writeln!(&mut variant_code, "            \"type\": {type_var_name},")?;
                            writeln!(&mut variant_code, "            \"payload\": payload_tmp,")?;
                            writeln!(&mut variant_code, "        }}")?;
                        } else {
                            writeln!(&mut variant_code, "        return {{")?;
                            writeln!(&mut variant_code, "            \"type\": {type_var_name},")?;
                            writeln!(&mut variant_code, "        }}")?;
                        }
                    }

                    writeln!(&mut result, "{}", variant_code)?;
                }
            }
            crate::ModelType::Struct(struct_def) => {
                writeln!(&mut result, "@dataclass")?;

                if let Some(virtual_name) = &struct_def.extend {
                    writeln!(&mut result, "class {}({}):", model.name, virtual_name)?;
                } else {
                    writeln!(&mut result, "class {}:", model.name)?;
                };

                let mut fields = vec![];
                if let Some(base) = &struct_def.extend {
                    let base_model = def.get_model(&base).unwrap();
                    match &base_model.type_ {
                        crate::ModelType::Virtual(struct_def) => {
                            fields.extend(struct_def.fields.clone());
                        }
                        _ => {
                            anyhow::bail!("only extend for virtual");
                        }
                    }
                }
                fields.extend(struct_def.fields.clone());

                if fields.is_empty() {
                    writeln!(&mut result, "    pass")?;
                } else {
                    for field in fields.iter() {
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
                let to_dict = generate_to_dict(&fields, &def)?;
                writeln!(&mut result, "{}", utils::indent(&to_dict, 1))?;

                writeln!(&mut result, "")?;
                let from_dict = generate_from_dict(&model.name, &fields, &def)?;
                writeln!(&mut result, "{}", utils::indent(&from_dict, 1))?;
            }

            crate::ModelType::Virtual(..) => {
                writeln!(&mut result, "class {model_name}(abc.ABC):")?;
                writeln!(&mut result, "    pass")?;
                writeln!(&mut result, "")?;
                writeln!(&mut result, "    @staticmethod")?;
                writeln!(&mut result, "    @abc.abstractmethod")?;
                writeln!(&mut result, "    def from_dict(d): pass")?;
                writeln!(&mut result, "")?;
                writeln!(&mut result, "    @abc.abstractmethod")?;
                writeln!(&mut result, "    def to_dict(self): pass")?;
            }

            crate::ModelType::NewType { inner_type } => {
                writeln!(
                    &mut result,
                    "{} = typing.Type[{}]",
                    model.name,
                    py_type(inner_type)
                )?;
            }

            crate::ModelType::Const { value_type, values } => {
                writeln!(&mut result, "class {model_name}(abc.ABC):")?;

                let value_type_py = match value_type {
                    crate::ConstType::I8 | crate::ConstType::I64 => "int",
                    crate::ConstType::String => "str",
                };

                for value in values.iter() {
                    let value_name = &value.name;
                    let value_literal = match value_type {
                        crate::ConstType::I8 | crate::ConstType::I64 => value.value.clone(),
                        crate::ConstType::String => format!("\"{}\"", value.value),
                    };

                    if let Some(desc) = &value.desc {
                        writeln!(&mut result, "    # {}", desc)?;
                    }
                    writeln!(
                        &mut result,
                        "    {value_name}: {value_type_py} = {value_literal}"
                    )?;
                }
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
        Type::Bool => "bool".into(),
        Type::I8 | Type::I64 => "int".into(),
        Type::F64 => "float".into(),
        Type::Bytes => "bytes".into(),
        Type::String => "str".into(),
        Type::List { item_type } => {
            format!("typing.List[{}]", py_type(item_type))
        }
        Type::Map { value_type } => format!("typing.Dict[str, {}]", py_type(value_type)),
        Type::Reference { target } => format!("\"{}\"", target),
    }
}

fn generate_to_dict(fields: &[FieldDef], def: &Definition) -> anyhow::Result<String> {
    let mut result = "".to_string();
    writeln!(&mut result, "def to_dict(self):")?;
    writeln!(&mut result, "    result = {{}}")?;

    for field in fields {
        writeln!(&mut result, "\n    # {}", field.name)?;

        match &field.type_ {
            Type::Bytes | Type::I64 | Type::I8 | Type::Bool | Type::F64 | Type::String => {
                writeln!(
                    &mut result,
                    "    result[\"{field_name}\"] = self.{field_name}",
                    field_name = field.name,
                )?;
            }

            ty => {
                // for List, Map, Reference
                let field_name = &field.name;
                let tmp_var_name = format!("{}_tmp", field.name);
                let to_dict =
                    to_dict_for_one_field(&ty, &format!("self.{field_name}"), &tmp_var_name, def)?;

                if field.required {
                    writeln!(&mut result, "{}", indent(&to_dict, 1))?;
                    writeln!(&mut result, "    result[\"{field_name}\"] = {tmp_var_name}")?;
                } else {
                    writeln!(&mut result, "    if self.{field_name} is None:")?;
                    writeln!(&mut result, "        result[\"{field_name}\"] = None")?;
                    writeln!(&mut result, "    else:")?;

                    writeln!(&mut result, "{}", indent(&to_dict, 2))?;
                    writeln!(
                        &mut result,
                        "        result[\"{field_name}\"] = {tmp_var_name}"
                    )?;
                }
            }
        }
    }

    writeln!(&mut result, "    return result")?;

    Ok(result)
}

fn to_dict_for_one_field(
    ty: &Type,
    in_expr: &str,
    out_var: &str,
    def: &Definition,
) -> anyhow::Result<String> {
    Ok(match ty {
        Type::Bool | Type::I8 | Type::I64 | Type::F64 | Type::String => {
            format!("{out_var} = {in_expr}")
        }
        Type::Bytes => {
            format!("{out_var} = list({in_expr})")
        }
        Type::List { item_type } => {
            let mut result = "".to_string();
            writeln!(&mut result, "{out_var} = []",)?;
            writeln!(&mut result, "for item in {in_expr}:",)?;
            let field_to_dict = to_dict_for_one_field(item_type, "item", "item_tmp", &def)?;
            writeln!(&mut result, "{}", indent(&field_to_dict, 1))?;
            writeln!(&mut result, "    {out_var}.append(item_tmp)")?;
            result
        }
        Type::Map { value_type } => {
            let mut result = "".to_string();
            writeln!(&mut result, "{out_var} = {{}}",)?;
            writeln!(&mut result, "for key, item in {in_expr}.items():")?;
            let field_to_dict = to_dict_for_one_field(value_type, "item", "item_tmp", &def)?;
            writeln!(&mut result, "{}", indent(&field_to_dict, 1))?;
            writeln!(&mut result, "    {out_var}[key] = item_tmp")?;
            result
        }
        Type::Reference { target } => {
            let target_model = def.get_model(target).unwrap();
            match &target_model.type_ {
                crate::ModelType::NewType { inner_type } => {
                    to_dict_for_one_field(&inner_type, in_expr, out_var, def)?
                }
                _ => {
                    format!("{out_var} = {in_expr}.to_dict()")
                }
            }
        }
    })
}

fn generate_from_dict(
    model_name: &str,
    fields: &[FieldDef],
    def: &Definition,
) -> anyhow::Result<String> {
    let mut code_block = "".to_string();

    let mut fields_init_codes = vec![];

    for field in fields {
        let field_name = &field.name;

        fields_init_codes.push(format!("{field_name} = {field_name},"));

        writeln!(&mut code_block, "\n# {field_name}")?;

        match &field.type_ {
            Type::Bool | Type::I8 | Type::I64 | Type::F64 | Type::String => {
                if field.required {
                    writeln!(&mut code_block, "{field_name} = d[\"{field_name}\"]")?;
                } else {
                    writeln!(
                        &mut code_block,
                        "{field_name} = d.get(\"{field_name}\", None)"
                    )?;
                }
            }
            ty @ Type::Bytes => {
                if field.required {
                    writeln!(&mut code_block, "{field_name} = bytes(d[\"{field_name}\"])")?;
                } else {
                    writeln!(&mut code_block, "{field_name} = None")?;
                    writeln!(&mut code_block, "if item := d.get(\"{field_name}\"):")?;

                    let from_dict_code_block =
                        from_dict_for_one_field(ty, "item", field_name, def)?;

                    writeln!(&mut code_block, "{}", indent(&from_dict_code_block, 1))?;
                }
            }
            ty => {
                if field.required {
                    let from_dict_code_block = from_dict_for_one_field(
                        ty,
                        &format!("d[\"{field_name}\"]"),
                        field_name,
                        def,
                    )?;

                    writeln!(&mut code_block, "{}", from_dict_code_block)?;
                } else {
                    writeln!(&mut code_block, "{field_name} = None")?;
                    writeln!(&mut code_block, "if item := d.get(\"{field_name}\"):")?;

                    let from_dict_code_block =
                        from_dict_for_one_field(ty, "item", field_name, def)?;

                    writeln!(&mut code_block, "{}", indent(&from_dict_code_block, 1))?;
                }
            }
        }
    }

    writeln!(&mut code_block, "return {model_name}(")?;
    for field_init_code in fields_init_codes {
        writeln!(&mut code_block, "{}", indent(&field_init_code, 1))?;
    }
    writeln!(&mut code_block, ")")?;

    let mut result = "".to_string();
    writeln!(&mut result, "@staticmethod")?;
    writeln!(&mut result, "def from_dict(d):")?;
    writeln!(&mut result, "{}", indent(&code_block, 1))?;

    Ok(result)
}

fn from_dict_for_one_field(
    ty: &Type,
    in_expr: &str,
    out_var: &str,
    def: &Definition,
) -> anyhow::Result<String> {
    Ok(match ty {
        Type::Bool | Type::I8 | Type::I64 | Type::F64 | Type::String => {
            format!("{out_var} = {in_expr}")
        }
        Type::Bytes => {
            format!("{out_var} = bytes({in_expr})")
        }
        Type::List { item_type } => {
            let mut result = "".to_string();
            writeln!(&mut result, "{out_var} = []")?;
            writeln!(&mut result, "for item in {in_expr}:")?;
            let from_dict_for_item = from_dict_for_one_field(item_type, "item", "item_tmp", def)?;
            writeln!(&mut result, "{}", indent(&from_dict_for_item, 1))?;
            writeln!(&mut result, "    {out_var}.append(item_tmp)")?;
            result
        }
        Type::Map { value_type } => {
            let mut result = "".to_string();
            writeln!(&mut result, "{out_var} = {{}}")?;
            writeln!(&mut result, "for key, item in {in_expr}.items():")?;
            let from_dict_for_item = from_dict_for_one_field(value_type, "item", "item_tmp", def)?;
            writeln!(&mut result, "{}", indent(&from_dict_for_item, 1))?;
            writeln!(&mut result, "    {out_var}[key] = item_tmp")?;
            result
        }
        Type::Reference { target } => {
            let target_model = def.get_model(target).unwrap();
            match &target_model.type_ {
                crate::ModelType::NewType { inner_type } => {
                    from_dict_for_one_field(&inner_type, in_expr, out_var, def)?
                }
                _ => format!("{out_var} = {target}.from_dict({in_expr})"),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_codegen() {
        let specs = &[
            (
                include_str!("fixtures/specs/const_i8.yaml"),
                include_str!("fixtures/py_dataclass/const_i8.py"),
            ),
            (
                include_str!("fixtures/specs/const_i64.yaml"),
                include_str!("fixtures/py_dataclass/const_i64.py"),
            ),
            (
                include_str!("fixtures/specs/const_string.yaml"),
                include_str!("fixtures/py_dataclass/const_string.py"),
            ),
        ];

        for (spec, expected) in specs.iter() {
            let def = serde_yaml::from_str::<Definition>(&spec).unwrap();
            let rendered = render(&def).unwrap();

            if rendered.ne(expected) {
                println!("=== rendered:\n{}", rendered.as_str().trim());
                println!("=== expected:\n{}", expected.trim());
                assert!(false, "code not match");
            }
        }
    }
}
