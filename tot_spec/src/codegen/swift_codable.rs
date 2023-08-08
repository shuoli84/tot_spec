use std::borrow::Cow;
use std::fmt::Write;
use std::path::PathBuf;

use crate::codegen::utils::folder_tree::FolderTree;
use crate::{Definition, FieldDef, Type, TypeReference};

use super::utils::{indent, multiline_prefix_with};

#[derive(Default)]
pub struct SwiftCodable {}

impl super::Codegen for SwiftCodable {
    fn load_from_folder(_folder: &PathBuf) -> anyhow::Result<Self> {
        Ok(Self::default())
    }

    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        use walkdir::WalkDir;

        std::fs::create_dir_all(output).unwrap();
        let mut spec_folder = FolderTree::new();

        for entry in WalkDir::new(folder) {
            let entry = entry.unwrap();
            let spec = entry.path();

            if !spec.is_file() {
                continue;
            }
            if !spec
                .extension()
                .map(|ext| ext == "yaml")
                .unwrap_or_default()
            {
                continue;
            }

            let relative_path = spec.strip_prefix(folder).unwrap();
            spec_folder.insert(relative_path);

            // now we get a file ends with yaml, build the output path
            // todo: how to map spec to output path is also codegen dependant, maybe move into core?
            let output = {
                let mut output = output.clone();
                output.push(relative_path);
                output.set_extension("swift");
                output
            };

            {
                println!("generating spec={spec:?} output={output:?}");
                let spec_content = std::fs::read_to_string(spec).unwrap();
                let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

                let parent_folder = output.parent().unwrap();
                std::fs::create_dir_all(parent_folder).unwrap();

                let code = render(&def)?;

                std::fs::write(&output, code).unwrap();
                println!("write output to {:?}", output);
            }
        }

        Ok(())
    }
}

/// render the definition to a swift file
fn render(def: &Definition) -> anyhow::Result<String> {
    let meta = def.get_meta("swift_codable");

    let package_name = meta
        .get("package_name")
        .map(|s| Cow::Borrowed(s))
        .unwrap_or(Cow::Owned("PACKAGE".to_string()));

    let mut result = "".to_string();
    writeln!(result, "import Foundation")?;

    writeln!(result, "")?;
    writeln!(result, "public enum ModelError: Error {{")?;
    writeln!(result, "    case Error")?;
    writeln!(result, "}}")?;

    for model in def.models.iter() {
        let model_name = &model.name;

        writeln!(result, "")?;
        if let Some(desc) = &model.desc {
            writeln!(result, "{}", multiline_prefix_with(desc, "// "))?;
        }
        match &model.type_ {
            crate::ModelType::Enum { variants } => {
                writeln!(result, "public enum {}: Codable {{", model.name)?;

                for variant in variants {
                    if let Some(payload) = &variant.payload_type {
                        writeln!(
                            result,
                            "    case {}({})",
                            variant.name,
                            swift_type(&payload, &package_name)
                        )?;
                    } else {
                        writeln!(result, "    case {}", variant.name,)?;
                    }
                }

                writeln!(result, "\n    // coding keys")?;
                writeln!(result, "    enum CodingKeys: String, CodingKey {{")?;
                writeln!(result, "        case type, payload")?;
                writeln!(result, "    }}")?;

                // decoder
                let decoder_code = {
                    let mut code_block = "".to_string();
                    writeln!(code_block, "// decoder")?;
                    writeln!(code_block, "public init(from decoder: Decoder) throws {{")?;
                    writeln!(
                        code_block,
                        "    let container = try decoder.container(keyedBy: CodingKeys.self)"
                    )?;
                    writeln!(
                        code_block,
                        "    let type = try container.decode(String.self, forKey: CodingKeys.type)"
                    )?;

                    writeln!(code_block, "    switch type {{")?;

                    let mut case_blocks = vec![];

                    for variant in variants.iter() {
                        let variant_name = &variant.name;

                        let mut case_block = "".to_string();

                        writeln!(case_block, "case \"{variant_name}\":")?;
                        if let Some(payload_type) = &variant.payload_type {
                            let payload_type = swift_type(&payload_type, &package_name);
                            writeln!(case_block, "    let payload = try container.decode({payload_type}.self, forKey:.payload)")?;
                            writeln!(case_block, "    self = .{variant_name}(payload)")?;
                        } else {
                            writeln!(case_block, "    self = .{variant_name}")?;
                        }

                        case_blocks.push(case_block);
                    }

                    {
                        let mut default_block = "".to_string();
                        writeln!(default_block, "default:")?;
                        writeln!(default_block, "    throw ModelError.Error")?;
                        case_blocks.push(default_block);
                    }

                    for case_block in case_blocks.into_iter() {
                        writeln!(code_block, "{}", indent(case_block.trim(), 2))?;
                    }

                    writeln!(code_block, "    }}")?;
                    writeln!(code_block, "}}")?;
                    code_block
                };
                writeln!(result, "")?;
                writeln!(result, "{}", indent(decoder_code.trim(), 1))?;

                // encoder
                let encoder_code = {
                    let mut code_block = "".to_string();
                    writeln!(code_block, "// encoder")?;
                    writeln!(
                        code_block,
                        "public func encode(to encoder: Encoder) throws {{"
                    )?;

                    let func_body = {
                        let mut func_body = "".to_string();
                        writeln!(
                            func_body,
                            "var container = encoder.container(keyedBy: CodingKeys.self)"
                        )?;

                        writeln!(func_body, "switch self {{")?;

                        for variant in variants.iter() {
                            let name = &variant.name;
                            let mut case_code = "".to_string();

                            if variant.payload_type.is_some() {
                                writeln!(case_code, "case let .{name}(payload):")?;

                                writeln!(
                                    case_code,
                                    "    try container.encode(\"{name}\", forKey: .type)"
                                )?;
                                writeln!(
                                    case_code,
                                    "    try container.encode(payload, forKey: .payload)"
                                )?;
                            } else {
                                writeln!(case_code, "case .{name}:")?;
                                writeln!(
                                    case_code,
                                    "    try container.encode(\"{name}\", forKey: .type)"
                                )?;
                            }

                            writeln!(func_body, "{}", indent(case_code.trim(), 1))?;
                        }

                        writeln!(func_body, "}}")?;

                        func_body
                    };

                    writeln!(code_block, "{}", indent(&func_body.trim(), 1))?;
                    writeln!(code_block, "}}")?;
                    code_block
                };
                writeln!(result, "")?;
                writeln!(result, "{}", indent(&encoder_code.trim(), 1))?;

                writeln!(result, "}}")?;
            }
            crate::ModelType::Struct(struct_def) => {
                let mut fields: Vec<FieldDef> = vec![];

                if let Some(base_type_ref) = &struct_def.extend {
                    let base = swift_type_for_type_ref(base_type_ref, &package_name);
                    writeln!(result, "public struct {}: Codable, {base} {{", model.name)?;
                    let base_model = def.get_model(&base).unwrap();
                    match &base_model.type_ {
                        crate::ModelType::Virtual(struct_def) => {
                            fields = struct_def.fields.clone();
                        }
                        _ => {
                            anyhow::bail!("extends only support virtual base");
                        }
                    }
                } else {
                    writeln!(result, "public struct {}: Codable {{", model.name)?;
                }

                fields.extend(struct_def.fields.clone());

                for field in fields.iter() {
                    let field_name = &field.name;
                    let mut field_type = swift_type(&field.type_, &package_name);
                    if !field.required {
                        field_type = format!("{field_type}?");
                    }

                    writeln!(result, "    public var {field_name}: {field_type}")?;
                }

                // generate member intializer, the default initializer is internal
                // we need to generate a public one
                let code_block = generate_memberwise_init(&fields, &package_name)?;
                writeln!(result, "")?;
                writeln!(result, "{}", indent(code_block.trim(), 1))?;

                writeln!(result, "}}")?;
            }
            crate::ModelType::Virtual(struct_def) => {
                writeln!(result, "public protocol {} {{", model.name)?;

                for field in struct_def.fields.iter() {
                    let field_name = &field.name;
                    let mut field_type = swift_type(&field.type_, &package_name);
                    if !field.required {
                        field_type = format!("{field_type}?");
                    }

                    writeln!(result, "    var {field_name}: {field_type} {{")?;
                    writeln!(result, "        get")?;
                    writeln!(result, "        set")?;
                    writeln!(result, "    }}")?;
                }

                writeln!(result, "}}")?;
            }
            crate::ModelType::NewType { inner_type } => {
                writeln!(
                    result,
                    "public typealias {} = {}",
                    model.name,
                    swift_type(inner_type, &package_name)
                )?;
            }

            crate::ModelType::Const { value_type, values } => {
                let swift_ty = match value_type {
                    crate::ConstType::I8 => "Int8",
                    crate::ConstType::I16 => "Int16",
                    crate::ConstType::I32 => "Int32",
                    crate::ConstType::I64 => "Int64",
                    crate::ConstType::String => "String",
                };

                writeln!(result, "public enum {model_name}: {swift_ty} {{",)?;
                for value in values.iter() {
                    let value_name = &value.name;
                    if let Some(desc) = &value.desc {
                        let comment = indent(multiline_prefix_with(desc, "// "), 1);
                        writeln!(result, "{comment}")?;
                    }

                    let value_literal = match &value.value {
                        crate::StringOrInteger::String(s) => format!("\"{s}\""),
                        crate::StringOrInteger::Integer(i) => i.to_string(),
                    };

                    writeln!(result, "    case {value_name} = {value_literal}",)?;
                }
                writeln!(result, "}}",)?;
            }
        }
    }

    Ok(result)
}

fn swift_type(ty: &Type, package_name: &str) -> String {
    match ty {
        Type::Bool => "Bool".into(),
        Type::I8 => "Int8".into(),
        Type::I16 => "Int16".into(),
        Type::I32 => "Int32".into(),
        Type::I64 => "Int64".into(),
        Type::F64 => "Float64".into(),
        Type::Bytes => "Data".into(),
        Type::String => "String".into(),
        Type::List { item_type } => {
            format!("[{}]", swift_type(item_type, package_name))
        }
        Type::Map { value_type } => {
            format!("[String:{}]", swift_type(value_type, package_name))
        }
        Type::Reference(type_ref) => swift_type_for_type_ref(type_ref, package_name),
        Type::Json => todo!(),
        Type::Decimal => todo!(),
        Type::BigInt => todo!(),
    }
}

fn swift_type_for_type_ref(type_ref: &TypeReference, package_name: &str) -> String {
    let TypeReference { namespace, target } = type_ref;
    if namespace.is_some() {
        todo!("swift codegen cross package reference is not supported yet")
    }
    format!("{}.{}", package_name, target)
}

fn generate_memberwise_init(fields: &[FieldDef], package_name: &str) -> anyhow::Result<String> {
    let mut code = "".to_string();

    let field_params = {
        let mut field_params = vec![];
        for field in fields.iter() {
            let field_name = &field.name;
            if field.required {
                field_params.push(format!(
                    "{field_name}: {}",
                    swift_type(&field.type_, package_name)
                ));
            } else {
                field_params.push(format!(
                    "{field_name}: {}? = nil",
                    swift_type(&field.type_, package_name)
                ));
            }
        }

        field_params.join(", ")
    };

    writeln!(code, "public init({field_params}) {{")?;

    for field in fields.iter() {
        let field_name = &field.name;
        writeln!(code, "    self.{field_name} = {field_name}",)?;
    }

    writeln!(code, "}}")?;

    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_codable() {
        let specs = &[
            (
                include_str!("fixtures/specs/const_i8.yaml"),
                include_str!("fixtures/swift_codable/const_i8.swift"),
            ),
            (
                include_str!("fixtures/specs/const_string.yaml"),
                include_str!("fixtures/swift_codable/const_string.swift"),
            ),
        ];

        for (spec, expected) in specs.iter() {
            let def = serde_yaml::from_str::<Definition>(&spec).unwrap();
            let rendered = render(&def).unwrap();

            pretty_assertions::assert_eq!(rendered.as_str().trim(), expected.trim());
        }
    }
}
