use crate::{Definition, FieldDef, StringOrInteger, Type, TypeReference};
use std::fmt::Write;

use super::utils::{self, indent, multiline_prefix_with};

pub fn render(def: &Definition) -> anyhow::Result<String> {
    let type_var_name = "type_";

    let mut result = String::new();

    writeln!(result, "from dataclasses import dataclass")?;
    writeln!(result, "import abc")?;
    writeln!(result, "import typing")?;

    writeln!(result, "")?;

    for model in def.models.iter() {
        let model_name = &model.name;

        writeln!(result, "")?;

        let comment = if let Some(desc) = &model.desc {
            multiline_prefix_with(desc, "# ")
        } else {
            model_name.clone()
        };
        writeln!(result, "{comment}")?;

        match &model.type_ {
            // python has no built in enum, so we generate base class
            // and each variants as a separate class
            crate::ModelType::Enum { variants } => {
                let enum_name = &model.name;
                writeln!(result, "class {enum_name}(abc.ABC):")?;
                writeln!(result, "    pass")?;

                // to_dict
                {
                    writeln!(result, "")?;
                    writeln!(result, "    @abc.abstractmethod")?;
                    writeln!(result, "    def to_dict(self):")?;
                    writeln!(result, "        pass")?;
                }

                // from_dict is the real impl, no variant sub class should provide
                // the impl
                let code_block = {
                    let mut code_block = "".to_string();
                    writeln!(code_block, "")?;
                    writeln!(code_block, "@staticmethod")?;
                    writeln!(code_block, "def from_dict(d):")?;
                    writeln!(code_block, "    {type_var_name} = d[\"type\"]")?;

                    for (variant_idx, variant) in variants.iter().enumerate() {
                        let variant_name = &variant.name;
                        let type_tag = variant_name.clone();
                        let variant_cls_name = format!("{enum_name}_{variant_name}");

                        if variant_idx == 0 {
                            writeln!(code_block, "    if {type_var_name} == \"{type_tag}\":")?;
                        } else {
                            writeln!(code_block, "    elif {type_var_name} == \"{type_tag}\":")?;
                        }

                        if let Some(payload_type) = &variant.payload_type {
                            writeln!(code_block, "        payload = d[\"payload\"]")?;

                            let payload_from_dict = from_dict_for_one_field(
                                payload_type,
                                "payload",
                                "payload_tmp",
                                def,
                            )?;
                            writeln!(code_block, "{}", indent(&payload_from_dict, 2))?;
                            writeln!(
                                code_block,
                                "        return {variant_cls_name}(payload=payload_tmp)"
                            )?;
                        } else {
                            writeln!(code_block, "        {variant_cls_name}()")?;
                        }
                    }

                    writeln!(code_block, "    else:")?;
                    writeln!(
                        code_block,
                        "        raise ValueError(f\"invalid type: {{{type_var_name}}}\")"
                    )?;

                    code_block
                };
                writeln!(result, "{}", indent(&code_block, 1))?;

                // generate sub class for each variant
                for variant in variants {
                    let variant_name = &variant.name;

                    let mut variant_code = "".to_string();
                    writeln!(variant_code, "")?;
                    writeln!(variant_code, "# variant {variant_name} for {model_name}",)?;
                    writeln!(variant_code, "@dataclass")?;
                    writeln!(
                        variant_code,
                        "class {model_name}_{variant_name}({model_name}):",
                    )?;
                    if let Some(payload_type) = &variant.payload_type {
                        writeln!(variant_code, "    payload: {}", py_type(&payload_type))?;
                    } else {
                        writeln!(variant_code, "    pass")?;
                    }

                    // to_dict
                    {
                        writeln!(variant_code, "")?;
                        writeln!(variant_code, "    def to_dict(self):")?;
                        writeln!(variant_code, "        {type_var_name} = \"{variant_name}\"")?;

                        if let Some(payload_type) = &variant.payload_type {
                            let payload_to_dict = to_dict_for_one_field(
                                &payload_type,
                                "self.payload",
                                "payload_tmp",
                                def,
                            )?;
                            writeln!(variant_code, "{}", indent(&payload_to_dict, 2))?;
                            writeln!(variant_code, "        return {{")?;
                            writeln!(variant_code, "            \"type\": {type_var_name},")?;
                            writeln!(variant_code, "            \"payload\": payload_tmp,")?;
                            writeln!(variant_code, "        }}")?;
                        } else {
                            writeln!(variant_code, "        return {{")?;
                            writeln!(variant_code, "            \"type\": {type_var_name},")?;
                            writeln!(variant_code, "        }}")?;
                        }
                    }

                    writeln!(result, "{}", variant_code)?;
                }
            }
            crate::ModelType::Struct(struct_def) => {
                writeln!(result, "@dataclass")?;

                if let Some(virtual_name) = &struct_def.extend {
                    writeln!(result, "class {}({}):", model.name, virtual_name)?;
                } else {
                    writeln!(result, "class {}:", model.name)?;
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
                    writeln!(result, "    pass")?;
                } else {
                    for field in fields.iter() {
                        if field.required {
                            writeln!(result, "    {}: {}", field.name, py_type_for_field(&field))?;
                        } else {
                            // for optional field, use None as default value
                            writeln!(
                                result,
                                "    {}: {} = None",
                                field.name,
                                py_type_for_field(&field)
                            )?;
                        }
                    }
                }

                writeln!(result, "")?;
                let to_dict = generate_to_dict(&fields, &def)?;
                writeln!(result, "{}", utils::indent(&to_dict, 1))?;

                writeln!(result, "")?;
                let from_dict = generate_from_dict(&model.name, &fields, &def)?;
                writeln!(result, "{}", utils::indent(&from_dict, 1))?;
            }

            crate::ModelType::Virtual(..) => {
                writeln!(result, "class {model_name}(abc.ABC):")?;
                writeln!(result, "    pass")?;
                writeln!(result, "")?;
                writeln!(result, "    @staticmethod")?;
                writeln!(result, "    @abc.abstractmethod")?;
                writeln!(result, "    def from_dict(d): pass")?;
                writeln!(result, "")?;
                writeln!(result, "    @abc.abstractmethod")?;
                writeln!(result, "    def to_dict(self): pass")?;
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
                writeln!(result, "class {model_name}(abc.ABC):")?;

                let value_type_py = match value_type {
                    crate::ConstType::I8
                    | crate::ConstType::I16
                    | crate::ConstType::I32
                    | crate::ConstType::I64 => "int",
                    crate::ConstType::String => "str",
                };

                for value in values.iter() {
                    let value_name = &value.name;
                    let value_literal = py_const_literal(&value.value);

                    if let Some(desc) = &value.desc {
                        let comment = indent(multiline_prefix_with(desc, "# "), 1);
                        writeln!(result, "{comment}")?;
                    }
                    writeln!(
                        result,
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
        Type::I8 | Type::I16 | Type::I32 | Type::I64 => "int".into(),
        Type::F64 => "float".into(),
        Type::Bytes => "bytes".into(),
        Type::String => "str".into(),
        Type::List { item_type } => {
            format!("typing.List[{}]", py_type(item_type))
        }
        Type::Map { value_type } => format!("typing.Dict[str, {}]", py_type(value_type)),
        Type::Reference(TypeReference {
            namespace: _,
            target,
        }) => format!("\"{}\"", target),
        Type::Json => {
            // now we just mark json as Any
            "typing.Any".to_string()
        }
    }
}

fn generate_to_dict(fields: &[FieldDef], def: &Definition) -> anyhow::Result<String> {
    let mut result = "".to_string();
    writeln!(result, "def to_dict(self):")?;
    writeln!(result, "    result = {{}}")?;

    for field in fields {
        writeln!(result, "\n    # {}", field.name)?;

        match &*field.type_ {
            Type::Bytes
            | Type::I64
            | Type::I8
            | Type::Bool
            | Type::F64
            | Type::String
            | Type::Json => {
                writeln!(
                    result,
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
                    writeln!(result, "{}", indent(&to_dict, 1))?;
                    writeln!(result, "    result[\"{field_name}\"] = {tmp_var_name}")?;
                } else {
                    writeln!(result, "    if self.{field_name} is None:")?;
                    writeln!(result, "        result[\"{field_name}\"] = None")?;
                    writeln!(result, "    else:")?;

                    writeln!(result, "{}", indent(&to_dict, 2))?;
                    writeln!(result, "        result[\"{field_name}\"] = {tmp_var_name}")?;
                }
            }
        }
    }

    writeln!(result, "    return result")?;

    Ok(result)
}

fn to_dict_for_one_field(
    ty: &Type,
    in_expr: &str,
    out_var: &str,
    def: &Definition,
) -> anyhow::Result<String> {
    Ok(match ty {
        Type::Bool | Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F64 | Type::String => {
            format!("{out_var} = {in_expr}")
        }
        Type::Bytes => {
            format!("{out_var} = list({in_expr})")
        }
        Type::List { item_type } => {
            let mut result = "".to_string();
            writeln!(result, "{out_var} = []",)?;
            writeln!(result, "for item in {in_expr}:",)?;
            let field_to_dict = to_dict_for_one_field(item_type, "item", "item_tmp", &def)?;
            writeln!(result, "{}", indent(&field_to_dict, 1))?;
            writeln!(result, "    {out_var}.append(item_tmp)")?;
            result
        }
        Type::Map { value_type } => {
            let mut result = "".to_string();
            writeln!(result, "{out_var} = {{}}",)?;
            writeln!(result, "for key, item in {in_expr}.items():")?;
            let field_to_dict = to_dict_for_one_field(value_type, "item", "item_tmp", &def)?;
            writeln!(result, "{}", indent(&field_to_dict, 1))?;
            writeln!(result, "    {out_var}[key] = item_tmp")?;
            result
        }
        Type::Reference(TypeReference { namespace, target }) => {
            if namespace.is_some() {
                unimplemented!()
            }

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
        Type::Json => {
            // for json type, it can be either dict, list, int, str, float, None, but it should not contain
            // user defined struct, so it should be fine that we just assign it to output dict
            format!("{out_var} = {in_expr}")
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

        writeln!(code_block, "\n# {field_name}")?;

        match &*field.type_ {
            Type::Bool | Type::I8 | Type::I64 | Type::F64 | Type::String => {
                if field.required {
                    writeln!(code_block, "{field_name} = d[\"{field_name}\"]")?;
                } else {
                    writeln!(code_block, "{field_name} = d.get(\"{field_name}\", None)")?;
                }
            }
            ty @ Type::Bytes => {
                if field.required {
                    writeln!(code_block, "{field_name} = bytes(d[\"{field_name}\"])")?;
                } else {
                    writeln!(code_block, "{field_name} = None")?;
                    writeln!(code_block, "if item := d.get(\"{field_name}\"):")?;

                    let from_dict_code_block =
                        from_dict_for_one_field(ty, "item", field_name, def)?;

                    writeln!(code_block, "{}", indent(&from_dict_code_block, 1))?;
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

                    writeln!(code_block, "{}", from_dict_code_block)?;
                } else {
                    writeln!(code_block, "{field_name} = None")?;
                    writeln!(code_block, "if item := d.get(\"{field_name}\"):")?;

                    let from_dict_code_block =
                        from_dict_for_one_field(ty, "item", field_name, def)?;

                    writeln!(code_block, "{}", indent(&from_dict_code_block, 1))?;
                }
            }
        }
    }

    writeln!(code_block, "return {model_name}(")?;
    for field_init_code in fields_init_codes {
        writeln!(code_block, "{}", indent(&field_init_code, 1))?;
    }
    writeln!(code_block, ")")?;

    let mut result = "".to_string();
    writeln!(result, "@staticmethod")?;
    writeln!(result, "def from_dict(d):")?;
    writeln!(result, "{}", indent(&code_block, 1))?;

    Ok(result)
}

fn from_dict_for_one_field(
    ty: &Type,
    in_expr: &str,
    out_var: &str,
    def: &Definition,
) -> anyhow::Result<String> {
    Ok(match ty {
        Type::Bool | Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F64 | Type::String => {
            format!("{out_var} = {in_expr}")
        }
        Type::Bytes => {
            format!("{out_var} = bytes({in_expr})")
        }
        Type::List { item_type } => {
            let mut result = "".to_string();
            writeln!(result, "{out_var} = []")?;
            writeln!(result, "for item in {in_expr}:")?;
            let from_dict_for_item = from_dict_for_one_field(item_type, "item", "item_tmp", def)?;
            writeln!(result, "{}", indent(&from_dict_for_item, 1))?;
            writeln!(result, "    {out_var}.append(item_tmp)")?;
            result
        }
        Type::Map { value_type } => {
            let mut result = "".to_string();
            writeln!(result, "{out_var} = {{}}")?;
            writeln!(result, "for key, item in {in_expr}.items():")?;
            let from_dict_for_item = from_dict_for_one_field(value_type, "item", "item_tmp", def)?;
            writeln!(result, "{}", indent(&from_dict_for_item, 1))?;
            writeln!(result, "    {out_var}[key] = item_tmp")?;
            result
        }
        Type::Reference(TypeReference { namespace, target }) => {
            if namespace.is_some() {
                unimplemented!()
            }

            let target_model = def.get_model(target).unwrap();
            match &target_model.type_ {
                crate::ModelType::NewType { inner_type } => {
                    from_dict_for_one_field(&inner_type, in_expr, out_var, def)?
                }
                _ => format!("{out_var} = {target}.from_dict({in_expr})"),
            }
        }
        Type::Json => {
            // for json type, it should be fine to just assign to property
            format!("{out_var} = {in_expr}")
        }
    })
}

fn py_const_literal(val: &StringOrInteger) -> String {
    match val {
        StringOrInteger::String(s) => format!("\"{s}\""),
        StringOrInteger::Integer(i) => i.to_string(),
    }
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
            (
                include_str!("fixtures/specs/json.yaml"),
                include_str!("fixtures/py_dataclass/json.py"),
            ),
        ];

        for (spec, expected) in specs.iter() {
            let def = serde_yaml::from_str::<Definition>(&spec).unwrap();
            let rendered = render(&def).unwrap();

            pretty_assertions::assert_eq!(rendered.as_str().trim(), expected.trim());
        }
    }
}
