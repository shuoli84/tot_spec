use crate::{Context, Definition, ModelDef, Type, TypeReference};
use std::{borrow::Cow, fmt::Write, path::PathBuf};

/// java does not export to a file, instead, it exports to a folder
pub fn render(def: &Definition, context: &Context, target_folder: &PathBuf) -> anyhow::Result<()> {
    std::fs::create_dir_all(target_folder)?;

    let package_name = def
        .get_meta("java_jackson")
        .get("package")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing package"))?;

    let mut package_folder = target_folder.to_owned();

    package_name.split('.').for_each(|c| package_folder.push(c));
    std::fs::create_dir_all(&package_folder)?;

    for model in def.models.iter() {
        let model_name = &model.name;
        let file_path = package_folder.join(format!("{model_name}.java"));
        let file_content = render_one(model, &package_name, def, context)?;

        std::fs::write(file_path, file_content)?;
    }

    Ok(())
}

pub fn render_one(
    model: &ModelDef,
    package_name: &str,
    def: &Definition,
    context: &Context,
) -> anyhow::Result<String> {
    let mut result = "".to_string();

    writeln!(result, "package {package_name};")?;
    writeln!(result, "import lombok.Data;")?;
    writeln!(result, "import java.util.*;")?;

    writeln!(result, "")?;

    let model_name = &model.name;

    match &model.type_ {
        crate::ModelType::Struct(st) => {
            // Data annotation makes the class a pojo
            writeln!(result, "@Data")?;

            match st.extend.as_ref() {
                Some(base) => {
                    if let Some(type_ref) = TypeReference::try_parse(base) {
                        let java_type = java_type_for_type_reference(&type_ref, def, context)?;
                        writeln!(
                            result,
                            "public class {name} extends {base} {{",
                            name = model.name,
                            base = java_type
                        )?;
                    }
                }
                None => {
                    writeln!(result, "public class {} {{", model.name)?;
                }
            }

            for field in st.fields.iter() {
                if let Some(desc) = &field.desc {
                    writeln!(result, "    // {}", desc)?;
                }
                writeln!(
                    result,
                    "    private {java_type} {name};",
                    java_type = java_type(&field.type_, def, context)?,
                    name = field.name
                )?;
            }

            writeln!(result, "}}")?;
        }
        crate::ModelType::Enum { variants } => {
            // Data annotation makes the class a pojo
            writeln!(
                result,
                "@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, property = \"type\")"
            )?;
            {
                writeln!(result, "@JsonSubTypes({{")?;

                for v in variants {
                    writeln!(
                            result,
                            "    @JsonSubTypes.Type(value = {model_name}.{name}.class, name = \"{name}\"),",
                            name = v.name
                        )?;
                }

                writeln!(result, "}})")?;
            }
            writeln!(result, "public abstract class {} {{", model.name)?;

            for (idx, v) in variants.iter().enumerate() {
                let variant_name = &v.name;

                writeln!(result, "    @Data")?;
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
                        if let Some(desc) = &field.desc {
                            writeln!(result, "    // {}", desc)?;
                        }
                        writeln!(
                            result,
                            "    private {java_type} {name};",
                            java_type = java_type(&field.type_, def, context)?,
                            name = field.name
                        )?;
                    }
                }
            }

            writeln!(result, "}}")?;
        }
        crate::ModelType::NewType { inner_type } => {
            writeln!(result, "public class {model_name} {{")?;
            let java_type = java_type(&inner_type, def, context)?;

            writeln!(result, "    private {java_type} value;",)?;

            writeln!(result, "")?;

            writeln!(result, "    @com.fasterxml.jackson.annotation.JsonCreator")?;
            writeln!(result, "    public {model_name}({java_type} value) {{")?;
            writeln!(result, "        this.value = value;")?;
            writeln!(result, "    }}")?;

            writeln!(result, "")?;

            writeln!(result, "    @com.fasterxml.jackson.annotation.JsonValue")?;
            writeln!(result, "    public {java_type} get_value() {{")?;
            writeln!(result, "        return value;")?;
            writeln!(result, "    }}")?;

            writeln!(result, "}}")?;
        }
        crate::ModelType::Const { value_type, values } => todo!(),
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

fn java_package_for_def(def: &Definition) -> String {
    let meta = def.get_meta("java_jackson");
    let package_name = meta
        .get("package")
        .map(|s| Cow::Borrowed(s))
        .unwrap_or(Cow::Owned("PACKAGE".to_string()));
    package_name.to_string()
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
        ];

        for (spec, package_folder) in specs.iter() {
            let context = Context::load_from_path(spec).unwrap();
            let def = context.load_from_yaml(spec).unwrap();
            render(&def, &context, &std::path::PathBuf::from(package_folder)).unwrap();
        }
    }
}
