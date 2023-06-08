use convert_case::Casing;

use crate::{
    ConstType, Context, Definition, FieldDef, ModelDef, StringOrInteger, Type, TypeReference,
};
use std::{borrow::Cow, fmt::Write, path::PathBuf};

use super::utils;

/// java does not export to a file, instead, it exports to a folder
pub fn render(def: &Definition, context: &Context, target_folder: &PathBuf) -> anyhow::Result<()> {
    std::fs::create_dir_all(target_folder)?;

    let package_name = def
        .get_meta("java_jackson")
        .get("package")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing package"))?;

    // if namespace_cls exists, then all models will be put into the namespace_cls
    let namespace_cls = def.get_meta("java_jackson").get("namespace_class").cloned();

    let mut package_folder = target_folder.to_owned();

    package_name.split('.').for_each(|c| package_folder.push(c));
    std::fs::create_dir_all(&package_folder)?;

    match namespace_cls {
        None => {
            for model in def.models.iter() {
                let model_name = &model.name;
                let file_path = package_folder.join(format!("{model_name}.java"));

                let mut result = "".to_string();

                writeln!(result, "package {package_name};")?;
                writeln!(result, "import lombok.*;")?;
                writeln!(result, "import java.util.*;")?;

                writeln!(result, "")?;

                let model_code = render_model(model, false, def, context)?;

                writeln!(result, "{}", model_code.trim_end())?;

                std::fs::write(file_path, result)?;
            }
        }

        Some(namespace_class) => {
            let file_path = package_folder.join(format!("{namespace_class}.java"));

            let mut result = "".to_string();

            writeln!(result, "package {package_name};")?;
            writeln!(result, "import lombok.*;")?;
            writeln!(result, "import java.util.*;")?;
            writeln!(result, "")?;

            writeln!(result, "public class {namespace_class} {{")?;

            // private constructor ensures this class is not instiatable
            writeln!(result, "    private {namespace_class}() {{}}")?;
            writeln!(result, "")?;

            for (idx, model) in def.models.iter().enumerate() {
                let model_code = render_model(model, true, def, context)?;
                let model_code = utils::indent(&model_code, 1);
                if idx + 1 < def.models.len() {
                    writeln!(result, "{}", model_code)?;
                } else {
                    writeln!(result, "{}", model_code.trim_end())?;
                }
            }

            writeln!(result, "}}")?;

            std::fs::write(file_path, result)?;
        }
    }

    Ok(())
}

pub fn render_model(
    model: &ModelDef,
    is_nested: bool,
    def: &Definition,
    context: &Context,
) -> anyhow::Result<String> {
    let mut result = "".to_string();

    let model_name = &model.name;

    if let Some(desc) = &model.desc {
        writeln!(result, "// {desc}")?;
    }

    let class_modifier = if is_nested { "static " } else { "" };

    let annotations = model
        .attribute("java_extra_annotation")
        .map(|a| {
            a.split(",")
                .map(|a| a.trim().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for annotation in annotations {
        writeln!(result, "@{annotation}")?;
    }

    match &model.type_ {
        crate::ModelType::Struct(st) => {
            // Data annotation makes the class a pojo
            writeln!(result, "@Data")?;
            writeln!(result, "@Builder")?;
            writeln!(result, "@AllArgsConstructor")?;
            writeln!(result, "@NoArgsConstructor")?;

            match st.extend.as_ref() {
                Some(base) => {
                    if let Some(type_ref) = TypeReference::try_parse(base) {
                        let java_type = java_type_for_type_reference(&type_ref, def, context)?;
                        writeln!(
                            result,
                            "public {class_modifier}class {name} extends {base} {{",
                            name = model.name,
                            base = java_type
                        )?;
                    }
                }
                None => {
                    writeln!(result, "public {class_modifier}class {} {{", model.name)?;
                }
            }

            for field in st.fields.iter() {
                result.push_str(&render_field(field, def, context)?);
            }

            writeln!(result, "}}")?;
        }
        crate::ModelType::Enum { variants } => {
            // Data annotation makes the class a pojo
            writeln!(
                result,
                "@com.fasterxml.jackson.annotation.JsonTypeInfo(use = com.fasterxml.jackson.annotation.JsonTypeInfo.Id.NAME, property = \"type\")"
            )?;
            {
                writeln!(result, "@com.fasterxml.jackson.annotation.JsonSubTypes({{")?;

                for v in variants {
                    writeln!(
                            result,
                            "    @com.fasterxml.jackson.annotation.JsonSubTypes.Type(value = {model_name}.{name}.class, name = \"{name}\"),",
                            name = v.name
                        )?;
                }

                writeln!(result, "}})")?;
            }
            writeln!(result, "public abstract class {} {{", model.name)?;

            for (idx, v) in variants.iter().enumerate() {
                let variant_name = &v.name;

                writeln!(result, "    @Data")?;
                writeln!(result, "    @Builder")?;
                writeln!(result, "    @AllArgsConstructor")?;
                writeln!(result, "    @NoArgsConstructor")?;
                writeln!(
                    result,
                    "    public static class {variant_name} extends {model_name} {{"
                )?;

                match v.payload_type.as_ref() {
                    Some(payload_type) => {
                        writeln!(
                            result,
                            "        private {} payload;",
                            java_type(payload_type, def, context)?,
                        )?;
                    }
                    None => todo!(),
                }

                writeln!(result, "    }}")?;

                if idx + 1 < variants.len() {
                    writeln!(result)?;
                }
            }

            writeln!(result, "}}")?;
        }
        crate::ModelType::Virtual(st) => {
            // Data annotation makes the class a pojo
            writeln!(result, "@Data")?;
            writeln!(result, "public abstract class {} {{", model.name)?;

            match st.extend.as_ref() {
                Some(_) => todo!(),
                None => {
                    for field in st.fields.iter() {
                        result.push_str(&render_field(field, def, context)?);
                    }
                }
            }

            writeln!(result, "}}")?;
        }
        crate::ModelType::NewType { inner_type } => {
            for annotation in ["Data", "Builder", "AllArgsConstructor", "NoArgsConstructor"] {
                writeln!(result, "@{annotation}")?;
            }

            writeln!(result, "public static class {model_name} {{")?;
            let java_type = java_type(&inner_type, def, context)?;

            writeln!(result, "    private {java_type} value;",)?;

            writeln!(result, "")?;

            writeln!(result, "    @com.fasterxml.jackson.annotation.JsonCreator")?;
            writeln!(result, "    public {model_name}({java_type} value) {{")?;
            writeln!(result, "        this.value = value;")?;
            writeln!(result, "    }}")?;

            writeln!(result, "")?;

            writeln!(result, "    @com.fasterxml.jackson.annotation.JsonValue")?;
            writeln!(result, "    public {java_type} getValue() {{")?;
            writeln!(result, "        return value;")?;
            writeln!(result, "    }}")?;

            writeln!(result, "}}")?;
        }
        crate::ModelType::Const { value_type, values } => {
            writeln!(result, "public {class_modifier}class {model_name} {{")?;
            let java_type = java_type_for_const(&value_type);

            for (idx, value) in values.iter().enumerate() {
                if let Some(desc) = &value.desc {
                    writeln!(result, "    // {desc}")?;
                }
                writeln!(
                    result,
                    "    public static final {java_type} {} = {};",
                    value.name.to_case(convert_case::Case::UpperSnake),
                    java_literal(&value.value)
                )?;

                if idx + 1 < values.len() {
                    writeln!(result, "")?;
                }
            }

            writeln!(result, "}}")?;
        }
    }

    Ok(result)
}

fn java_type(ty: &Type, def: &Definition, context: &Context) -> anyhow::Result<String> {
    Ok(match ty {
        Type::Bool => "Boolean".into(),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 => "Integer".into(),
        Type::F64 => "Double".into(),
        Type::Bytes => "byte[]".into(),
        Type::String => "String".into(),
        Type::List { item_type } => {
            format!("List<{}>", java_type(item_type, def, context)?)
        }
        Type::Map { value_type } => {
            format!("Map<String, {}>", java_type(value_type, def, context)?)
        }
        Type::Reference(type_ref) => java_type_for_type_reference(type_ref, def, context)?,
        Type::Json => "com.fasterxml.jackson.databind.JsonNode".to_string(),
        Type::Decimal => "java.math.BigDecimal".into(),
        Type::BigInt => "java.math.BigInteger".into(),
    })
}

fn java_type_for_type_reference(
    type_ref: &TypeReference,
    def: &Definition,
    context: &Context,
) -> anyhow::Result<String> {
    let TypeReference { namespace, target } = type_ref;
    let fqdn_target = match namespace {
        Some(namespace) => {
            let include_def = context.load_include_def(namespace, def)?;
            let package = java_package_for_def(&include_def);
            format!("{package}.{target}")
        }
        None => {
            let package = java_package_for_def(def);
            format!("{package}.{target}")
        }
    };

    Ok(fqdn_target)
}

fn java_type_for_const(ty: &ConstType) -> &'static str {
    match ty {
        ConstType::I8 | ConstType::I16 | ConstType::I32 | ConstType::I64 => "Integer",
        ConstType::String => "String",
    }
}

fn java_literal(val: &StringOrInteger) -> String {
    match val {
        StringOrInteger::String(val) => format!("\"{}\"", val.replace('"', "\"")),
        StringOrInteger::Integer(val) => format!("{val}"),
    }
}

fn java_package_for_def(def: &Definition) -> String {
    let meta = def.get_meta("java_jackson");
    let package_name = meta
        .get("package")
        .map(|s| Cow::Borrowed(s))
        .unwrap_or(Cow::Owned("PACKAGE".to_string()));

    match meta.get("namespace_class") {
        None => package_name.to_string(),
        Some(namespace_class) => {
            format!("{}.{}", package_name, namespace_class)
        }
    }
}

fn render_field(field: &FieldDef, def: &Definition, context: &Context) -> anyhow::Result<String> {
    let mut result = "".to_string();
    if let Some(desc) = &field.desc {
        writeln!(result, "    // {}", desc)?;
    }

    let java_field_name = field.name.to_case(convert_case::Case::Camel);
    let need_rename = java_field_name.ne(&field.name);

    if need_rename {
        writeln!(
            result,
            "    @com.fasterxml.jackson.annotation.JsonProperty(\"{}\")",
            field.name
        )?;
    }

    writeln!(
        result,
        "    private {java_type} {name};",
        java_type = java_type(&field.type_, def, context)?,
        name = java_field_name
    )?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_jackson() {
        let specs = &[
            (
                "src/codegen/fixtures/specs/include_test.yaml",
                "src/codegen/fixtures/java_jackson/include_test/",
            ),
            (
                "src/codegen/fixtures/specs/simple_struct.yaml",
                "src/codegen/fixtures/java_jackson/simple_struct",
            ),
            (
                "src/codegen/fixtures/specs/enum.yaml",
                "src/codegen/fixtures/java_jackson/enum",
            ),
            (
                "src/codegen/fixtures/specs/extend.yaml",
                "src/codegen/fixtures/java_jackson/extend",
            ),
            (
                "src/codegen/fixtures/specs/json.yaml",
                "src/codegen/fixtures/java_jackson/json",
            ),
            (
                "src/codegen/fixtures/specs/new_type.yaml",
                "src/codegen/fixtures/java_jackson/new_type",
            ),
            (
                "src/codegen/fixtures/specs/const_string.yaml",
                "src/codegen/fixtures/java_jackson/const_string",
            ),
            (
                "src/codegen/fixtures/specs/const_i64.yaml",
                "src/codegen/fixtures/java_jackson/const_i64",
            ),
            (
                "src/codegen/fixtures/specs/decimal.yaml",
                "src/codegen/fixtures/java_jackson/decimal",
            ),
            (
                "src/codegen/fixtures/specs/bigint.yaml",
                "src/codegen/fixtures/java_jackson/bigint",
            ),
            (
                "src/codegen/fixtures/specs/java_namespace.yaml",
                "src/codegen/fixtures/java_jackson/namespace",
            ),
        ];

        for (spec, package_folder) in specs.iter() {
            let context = Context::load_from_path(spec).unwrap();
            let def = context.load_from_yaml(spec).unwrap();
            render(&def, &context, &std::path::PathBuf::from(package_folder)).unwrap();
        }
    }
}
