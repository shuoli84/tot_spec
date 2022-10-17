use std::borrow::Cow;
use std::fmt::Write;

use crate::{Definition, FieldDef, Type};

use super::utils::indent;

/// render the definition to a swift file
pub fn render(def: &Definition) -> anyhow::Result<String> {
    let meta = def.get_meta("swift_codable");

    let package_name = meta
        .get("package_name")
        .map(|s| Cow::Borrowed(s))
        .unwrap_or(Cow::Owned("PACKAGE".to_string()));

    let mut result = "".to_string();
    writeln!(&mut result, "import Foundation")?;

    writeln!(&mut result, "")?;
    writeln!(&mut result, "public enum ModelError: Error {{")?;
    writeln!(&mut result, "    case Error")?;
    writeln!(&mut result, "}}")?;

    for model in def.models.iter() {
        let model_name = &model.name;

        writeln!(&mut result, "\n // {model_name}")?;
        match &model.type_ {
            crate::ModelType::Enum { variants } => {
                writeln!(&mut result, "public enum {}: Codable {{", model.name)?;

                for variant in variants {
                    if let Some(payload) = &variant.payload_type {
                        writeln!(
                            &mut result,
                            "    case {}({})",
                            variant.name,
                            swift_type(&payload, &package_name)
                        )?;
                    } else {
                        writeln!(&mut result, "    case {}", variant.name,)?;
                    }
                }

                writeln!(&mut result, "\n    // coding keys")?;
                writeln!(&mut result, "    enum CodingKeys: String, CodingKey {{")?;
                writeln!(&mut result, "        case type, payload")?;
                writeln!(&mut result, "    }}")?;

                // decoder
                let decoder_code = {
                    let mut code_block = "".to_string();
                    writeln!(&mut code_block, "\n// decoder")?;
                    writeln!(
                        &mut code_block,
                        "public init(from decoder: Decoder) throws {{"
                    )?;
                    writeln!(
                        &mut code_block,
                        "    let container = try decoder.container(keyedBy: CodingKeys.self)"
                    )?;
                    writeln!(
                        &mut code_block,
                        "    let type = try container.decode(String.self, forKey: CodingKeys.type)"
                    )?;

                    writeln!(&mut code_block, "    switch type {{")?;

                    let mut case_blocks = vec![];

                    for variant in variants.iter() {
                        let variant_name = &variant.name;

                        let mut case_block = "".to_string();

                        writeln!(&mut case_block, "case \"{variant_name}\":")?;
                        if let Some(payload_type) = &variant.payload_type {
                            let payload_type = swift_type(&payload_type, &package_name);
                            writeln!(&mut case_block, "    let payload = try container.decode({payload_type}.self, forKey:.payload)")?;
                            writeln!(&mut case_block, "    self = .{variant_name}(payload)")?;
                        } else {
                            writeln!(&mut case_block, "    self = .{variant_name}")?;
                        }

                        case_blocks.push(case_block);
                    }

                    {
                        let mut default_block = "".to_string();
                        writeln!(&mut default_block, "default:")?;
                        writeln!(&mut default_block, "    throw ModelError.Error")?;
                        case_blocks.push(default_block);
                    }

                    for case_block in case_blocks.into_iter() {
                        writeln!(&mut code_block, "{}", indent(&case_block, 2))?;
                    }

                    writeln!(&mut code_block, "    }}")?;
                    writeln!(&mut code_block, "}}")?;
                    code_block
                };
                writeln!(&mut result, "{}", indent(&decoder_code, 1))?;

                // encoder
                let encoder_code = {
                    let mut code_block = "".to_string();
                    writeln!(&mut code_block, "\n// encoder")?;
                    writeln!(
                        &mut code_block,
                        "public func encode(to encoder: Encoder) throws {{"
                    )?;

                    let func_body = {
                        let mut func_body = "".to_string();
                        writeln!(
                            &mut func_body,
                            "var container = encoder.container(keyedBy: CodingKeys.self)"
                        )?;

                        writeln!(&mut func_body, "switch self {{")?;

                        for variant in variants.iter() {
                            let name = &variant.name;
                            let mut case_code = "".to_string();

                            if variant.payload_type.is_some() {
                                writeln!(&mut case_code, "case let .{name}(payload):")?;

                                writeln!(
                                    &mut case_code,
                                    "    try container.encode(\"{name}\", forKey: .type)"
                                )?;
                                writeln!(
                                    &mut case_code,
                                    "    try container.encode(payload, forKey: .payload)"
                                )?;
                            } else {
                                writeln!(&mut case_code, "case .{name}:")?;
                                writeln!(
                                    &mut case_code,
                                    "    try container.encode(\"{name}\", forKey: .type)"
                                )?;
                            }

                            writeln!(&mut func_body, "{}", indent(&case_code, 1))?;
                        }

                        writeln!(&mut func_body, "}}")?;

                        func_body
                    };

                    writeln!(&mut code_block, "{}", indent(&func_body, 1))?;
                    writeln!(&mut code_block, "}}")?;
                    code_block
                };
                writeln!(&mut result, "{}", indent(&encoder_code, 1))?;

                writeln!(&mut result, "}}")?;
            }
            crate::ModelType::Struct(struct_def) => {
                let mut fields: Vec<FieldDef> = vec![];

                if let Some(base) = &struct_def.extend {
                    writeln!(
                        &mut result,
                        "public struct {}: Codable, {base} {{",
                        model.name
                    )?;
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
                    writeln!(&mut result, "public struct {}: Codable {{", model.name)?;
                }

                fields.extend(struct_def.fields.clone());

                for field in fields.iter() {
                    let field_name = &field.name;
                    let mut field_type = swift_type(&field.type_, &package_name);
                    if !field.required {
                        field_type = format!("{field_type}?");
                    }

                    writeln!(&mut result, "    public var {field_name}: {field_type}")?;
                }

                // generate member intializer, the default initializer is internal
                // we need to generate a public one
                let code_block = generate_memberwise_init(&fields, &package_name)?;
                writeln!(&mut result, "{}", indent(&code_block, 1))?;

                writeln!(&mut result, "}}")?;
            }
            crate::ModelType::Virtual(struct_def) => {
                writeln!(&mut result, "public protocol {} {{", model.name)?;

                for field in struct_def.fields.iter() {
                    let field_name = &field.name;
                    let mut field_type = swift_type(&field.type_, &package_name);
                    if !field.required {
                        field_type = format!("{field_type}?");
                    }

                    writeln!(&mut result, "    var {field_name}: {field_type} {{")?;
                    writeln!(&mut result, "        get")?;
                    writeln!(&mut result, "        set")?;
                    writeln!(&mut result, "    }}")?;
                }

                writeln!(&mut result, "}}")?;
            }
            crate::ModelType::NewType { inner_type } => {
                writeln!(
                    &mut result,
                    "public typealias {} = {}",
                    model.name,
                    swift_type(inner_type, &package_name)
                )?;
            }
        }
    }

    Ok(result)
}

fn swift_type(ty: &Type, package_name: &str) -> String {
    match ty {
        Type::Bool => "Bool".into(),
        Type::I8 => "Int8".into(),
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
        Type::Reference { target } => {
            format!("{}.{}", package_name, target)
        }
    }
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

    writeln!(&mut code, "public init({field_params}) {{")?;

    for field in fields.iter() {
        let field_name = &field.name;
        writeln!(&mut code, "    self.{field_name} = {field_name}",)?;
    }

    writeln!(&mut code, "}}")?;

    Ok(code)
}
