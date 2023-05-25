use crate::{Context, Definition, Type};
use std::{borrow::Cow, fmt::Write};

pub fn render(def: &Definition, context: &Context) -> anyhow::Result<String> {
    let meta = def.get_meta("java_jackson");
    let package_name = meta
        .get("package")
        .map(|s| Cow::Borrowed(s))
        .unwrap_or(Cow::Owned("PACKAGE".to_string()));

    let mut result = "".to_string();

    writeln!(result, "package {package_name};")?;
    writeln!(result, "import lombok.Data;")?;
    writeln!(result, "import java.util.*;")?;

    for model in def.models.iter() {
        writeln!(result, "")?;

        let model_name = &model.name;

        match &model.type_ {
            crate::ModelType::Struct(st) => {
                // Data annotation makes the class a pojo
                writeln!(result, "@Data")?;
                writeln!(result, "public class {} {{", model.name)?;

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
            crate::ModelType::Virtual(_) => todo!(),
            crate::ModelType::NewType { inner_type } => todo!(),
            crate::ModelType::Const { value_type, values } => todo!(),
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
        Type::Reference { namespace, target } => {
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

            fqdn_target
        }
        Type::Json => todo!(),
    })
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
                include_str!("fixtures/java_jackson/include_test.java"),
            ),
            (
                "src/codegen/fixtures/specs/simple_struct.yaml",
                include_str!("fixtures/java_jackson/simple_struct.java"),
            ),
            (
                "src/codegen/fixtures/specs/enum.yaml",
                include_str!("fixtures/java_jackson/enum.java"),
            ),
        ];

        for (spec, expected) in specs.iter() {
            let context = Context::load_from_path(spec).unwrap();
            let def = context.load_from_yaml(spec).unwrap();
            let rendered = render(&def, &context).unwrap();

            pretty_assertions::assert_eq!(rendered.as_str().trim(), expected.trim());
        }
    }
}
