use super::Codegen;
use crate::codegen::context::Context;
use crate::{FieldDef, MethodDef, ModelDef, ModelType, Type, TypeReference};
use anyhow::anyhow;
use indexmap::IndexMap;
use openapiv3::{
    AdditionalProperties, Components, Example, Info, MediaType, OpenAPI, Operation, PathItem,
    ReferenceOr, RequestBody, Response, Responses, Schema, SchemaData, SchemaKind,
};
use serde::{Deserialize, Serialize};
use std::path::{Component, PathBuf};

#[derive(Default, Debug, Deserialize, Serialize)]
struct CodegenConfig {
    /// title
    title: String,

    /// description
    description: Option<String>,

    /// servers
    servers: Vec<openapiv3::Server>,

    /// method related config
    method: MethodConfig,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct MethodConfig {
    #[serde(default)]
    spec_as_method: SpecAsMethodConfig,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct SpecAsMethodConfig {
    #[serde(default = "serde_default::bool_false")]
    enable: bool,

    #[serde(default = "serde_default::default_path_separator")]
    path_separator: String,

    #[serde(default = "serde_default::default_request_model")]
    request_model: String,

    #[serde(default = "serde_default::default_response_model")]
    response_model: String,

    /// the path to retrieve method desc
    desc_path: Option<String>,
}

mod serde_default {
    pub(super) fn bool_false() -> bool {
        false
    }

    pub(super) fn default_path_separator() -> String {
        "/".into()
    }

    pub(super) fn default_request_model() -> String {
        "Request".into()
    }

    pub(super) fn default_response_model() -> String {
        "Response".into()
    }
}

#[derive(Default)]
pub struct Swagger {
    skip_failed: bool,
}

impl Swagger {
    fn load_config(config_file: &PathBuf) -> anyhow::Result<Option<CodegenConfig>> {
        if !config_file.exists() {
            return Ok(None);
        }

        let config_content = std::fs::read_to_string(config_file)
            .map_err(|_| anyhow::anyhow!("not able to read spec_config.yaml from folder"))?;
        let config_value =
            serde_yaml::from_str::<serde_json::Map<String, serde_json::Value>>(&config_content)?;
        let Some(codegen_value) = config_value.get("codegen") else {
            return Ok(None)
        };

        assert!(codegen_value.is_object());

        let Some(swagger_value) = codegen_value.as_object().unwrap().get("swagger") else {
            return Ok(None)
        };

        let swagger_config = serde_json::from_value::<CodegenConfig>(swagger_value.to_owned())?;

        Ok(Some(swagger_config))
    }
}

impl Codegen for Swagger {
    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        // load codegen config from spec_config.yaml file
        let config = Swagger::load_config(&folder.join("spec_config.yaml"))?.unwrap_or_default();

        let context = Context::new_from_folder(folder)?;

        let mut openapi_spec = OpenAPI {
            openapi: "3.0.0".to_string(),
            servers: config.servers.clone(),
            ..OpenAPI::default()
        };

        // load info from config
        openapi_spec.info = Info {
            title: config.title.clone(),
            description: config.description.clone(),
            terms_of_service: None,
            contact: None,
            license: None,
            version: "".to_string(),
            ..Default::default()
        };
        openapi_spec.components = Some(Components::default());

        for (spec, _) in context.iter_specs() {
            println!("swagger rendering {spec:?}");
            match self.render_one_spec(spec, &context, &mut openapi_spec, &config) {
                Ok(_) => continue,
                Err(_) if self.skip_failed => continue,
                Err(e) => return Err(e),
            }
        }

        let output_file = output.join("openapi.yaml");
        let yaml_str = serde_yaml::to_string(&openapi_spec)?;

        std::fs::write(&output_file, yaml_str)?;

        Ok(())
    }
}

impl Swagger {
    fn render_one_spec(
        &self,
        spec: &PathBuf,
        context: &Context,
        openapi_spec: &mut OpenAPI,
        config: &CodegenConfig,
    ) -> anyhow::Result<()> {
        let def = context.get_definition(spec)?;

        let mut methods = def.methods.clone();

        // construct methods dynamically
        if config.method.spec_as_method.enable {
            let mut method_desc: Option<String> = None;
            if let Some(desc_path) = &config.method.spec_as_method.desc_path {
                let mut components = desc_path.split('.');
                let c1 = components
                    .next()
                    .expect("path should in format meta_name.field_name");
                let c2 = components
                    .next()
                    .expect("path should in format meta_name.field_name");

                if let Some(desc) = def.get_meta(c1).get(c2) {
                    method_desc = desc.clone().into();
                }
            }

            let components = to_components(spec);
            let method_name = components.join(&config.method.spec_as_method.path_separator);

            if def
                .get_model(&config.method.spec_as_method.request_model)
                .is_some()
            {
                let method_value = serde_json::json!({
                    "name": method_name,
                    "desc": method_desc,
                    "request": config.method.spec_as_method.request_model,
                    "response": config.method.spec_as_method.response_model,
                });
                methods.push(serde_json::from_value::<MethodDef>(method_value)?);
            }
        }

        for method in &methods {
            let method_name = method.name.clone();

            let mut response_map = IndexMap::default();
            response_map.insert(
                openapiv3::StatusCode::Code(200),
                ReferenceOr::Item(Response {
                    description: "".to_string(),
                    content: {
                        let mut content_map = IndexMap::new();
                        content_map.insert(
                            "application/json".into(),
                            MediaType {
                                schema: Some(type_to_schema(
                                    &Type::Reference(method.response.0.clone()),
                                    true,
                                    spec,
                                    context,
                                )?),
                                example: None,
                                examples: Default::default(),
                                ..Default::default()
                            },
                        );
                        content_map
                    },
                    ..Default::default()
                }),
            );

            let path_item = ReferenceOr::Item(PathItem {
                post: Some(Operation {
                    summary: method.name.to_string().into(),
                    description: method.desc.clone(),
                    request_body: Some(ReferenceOr::Item(RequestBody {
                        description: None,
                        content: {
                            let mut content_map = IndexMap::new();
                            content_map.insert(
                                "application/json".into(),
                                MediaType {
                                    schema: Some(type_to_schema(
                                        &Type::Reference(method.request.0.clone()),
                                        true,
                                        spec,
                                        context,
                                    )?),
                                    examples: {
                                        let request_model_def = context
                                            .get_model_def_for_reference(&method.request.0, spec)?;

                                        let examples = load_json_examples(request_model_def)?;

                                        examples
                                            .into_iter()
                                            .map(|(k, v)| {
                                                (
                                                    k,
                                                    ReferenceOr::Item(Example {
                                                        value: Some(v),
                                                        ..Default::default()
                                                    }),
                                                )
                                            })
                                            .collect()
                                    },
                                    ..Default::default()
                                },
                            );
                            content_map
                        },
                        required: true,
                        ..Default::default()
                    })),
                    responses: Responses {
                        default: None,
                        responses: response_map,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                servers: vec![],
                ..Default::default()
            });

            openapi_spec
                .paths
                .paths
                .insert(format!("/{method_name}"), path_item);
        }

        for model in def.models.iter() {
            let model_desc = model.desc.clone();
            let model_name = &model.name;

            let schema = match &model.type_ {
                ModelType::Struct(st_) => {
                    let mut object_type = openapiv3::ObjectType::default();

                    let properties = fields_to_properties(&st_.fields, spec, context)?;
                    for (name, property_schema) in properties {
                        object_type.properties.insert(name, property_schema);
                    }

                    let example = load_one_json_example(model)?;

                    ReferenceOr::Item(Schema {
                        schema_kind: SchemaKind::Type(openapiv3::Type::Object(object_type)),
                        schema_data: SchemaData {
                            description: model_desc,
                            example,
                            ..Default::default()
                        },
                    })
                }
                ModelType::Enum { ref variants } => {
                    let mut variant_schemas = vec![];
                    for variant in variants.iter() {
                        // todo: enum variant embeded should converge to a separate model def

                        let payload_type = variant
                            .payload_type
                            .as_ref()
                            .ok_or_else(|| anyhow!("swagger enum now only support payload_type"))?;
                        let variant_schema = type_to_schema(&payload_type.0, true, spec, context)?;
                        variant_schemas.push(variant_schema);
                    }

                    ReferenceOr::Item(Schema {
                        schema_kind: SchemaKind::OneOf {
                            one_of: variant_schemas,
                        },
                        schema_data: SchemaData {
                            title: Some(model_fqdn(spec, model_name)),
                            description: model_desc,
                            ..Default::default()
                        },
                    })
                }
                ModelType::Virtual(_) => {
                    continue;
                }
                ModelType::NewType { inner_type } => {
                    let inner_type = &inner_type.as_ref().0;
                    type_to_schema(inner_type, true, spec, context)?
                }
                ModelType::Const { .. } => {
                    continue;
                }
            };

            openapi_spec
                .components
                .as_mut()
                .unwrap()
                .schemas
                .insert(model_fqdn(spec, model_name), schema);
        }

        Ok(())
    }
}

fn type_to_schema(
    ty_: &Type,
    required: bool,
    spec_path: &PathBuf,
    context: &Context,
) -> anyhow::Result<ReferenceOr<Schema>> {
    let schema_kind = match ty_ {
        Type::Bool => SchemaKind::Type(openapiv3::Type::Boolean {}),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 => {
            let number_format = if matches!(ty_, Type::I64) {
                openapiv3::IntegerFormat::Int64
            } else {
                openapiv3::IntegerFormat::Int32
            };

            SchemaKind::Type(openapiv3::Type::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(number_format),
                ..Default::default()
            }))
        }
        Type::F64 => SchemaKind::Type(openapiv3::Type::Number(openapiv3::NumberType {
            format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::NumberFormat::Double),
            ..Default::default()
        })),
        Type::Decimal | Type::BigInt | Type::Bytes | Type::String => {
            SchemaKind::Type(openapiv3::Type::String(openapiv3::StringType {
                format: Default::default(),
                ..Default::default()
            }))
        }
        Type::List { item_type } => {
            let item_type_ref = &item_type.as_ref().0;
            let item_schema = match type_to_schema(item_type_ref, false, spec_path, context)? {
                ReferenceOr::Reference { reference } => ReferenceOr::Reference { reference },
                ReferenceOr::Item(item) => ReferenceOr::Item(Box::new(item)),
            };

            SchemaKind::Type(openapiv3::Type::Array(openapiv3::ArrayType {
                items: Some(item_schema),
                min_items: None,
                max_items: None,
                unique_items: false,
            }))
        }
        Type::Map { .. } => SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
            additional_properties: Some(AdditionalProperties::Any(true)),
            ..Default::default()
        })),
        Type::Json => SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
            additional_properties: Some(AdditionalProperties::Any(true)),
            ..Default::default()
        })),
        Type::Reference(TypeReference { namespace, target }) => {
            let spec = match namespace {
                None => spec_path.to_owned(),
                Some(namespace) => {
                    let include_path = context.get_include_path(namespace, spec_path)?;
                    let include_def = context.get_definition(&include_path)?;

                    let _ = include_def
                        .get_model(&target)
                        .ok_or_else(|| anyhow!("Not able to load model {namespace}.{target}"))?;

                    include_path
                }
            };

            return Ok(ReferenceOr::Reference {
                reference: format!("#/components/schemas/{}", model_fqdn(&spec, target)),
            });
        }
    };

    Ok(ReferenceOr::Item(Schema {
        schema_kind,
        schema_data: SchemaData {
            nullable: !required,
            example: None,
            title: None,
            description: format!("{ty_:?}").into(),
            ..Default::default()
        },
    }))
}

fn model_fqdn(spec_path: &PathBuf, model_name: &str) -> String {
    assert!(spec_path.is_relative());

    let components = to_components(spec_path);
    let type_path_prefix = components.join("_");

    if !type_path_prefix.is_empty() {
        format!("{type_path_prefix}_{model_name}")
    } else {
        model_name.to_string()
    }
}

fn fields_to_properties(
    fields: &[FieldDef],
    spec: &PathBuf,
    context: &Context,
) -> anyhow::Result<Vec<(String, ReferenceOr<Box<Schema>>)>> {
    let mut properties = vec![];

    for field in fields.iter() {
        let field_schema = type_to_schema(&field.type_, field.required, spec, context)?;
        properties.push((
            field.name.to_string(),
            match field_schema {
                ReferenceOr::Reference { reference } => ReferenceOr::Reference { reference },
                ReferenceOr::Item(item) => ReferenceOr::boxed_item(item),
            },
        ));
    }

    Ok(properties)
}

fn to_components(path: &PathBuf) -> Vec<String> {
    assert!(path.is_relative());

    let components = path.components().collect::<Vec<_>>();
    components
        .iter()
        .map(|c| match c {
            Component::Normal(name) => {
                let name = name.to_string_lossy().to_string();
                name.strip_suffix(".yaml")
                    .map(|s| s.to_string())
                    .unwrap_or(name)
            }
            _ => {
                unimplemented!()
            }
        })
        .collect::<Vec<_>>()
}

/// load the first json example defined in ModelDef
fn load_one_json_example(model_def: &ModelDef) -> anyhow::Result<Option<serde_json::Value>> {
    model_def
        .examples
        .iter()
        .filter(|e| e.format.eq("json"))
        .nth(0)
        .map(|e| {
            let example_value = serde_json::from_str::<serde_json::Value>(e.value.as_str())?;
            Ok(example_value)
        })
        .transpose()
}

/// load the first json example defined in ModelDef
fn load_json_examples(model_def: &ModelDef) -> anyhow::Result<IndexMap<String, serde_json::Value>> {
    model_def
        .examples
        .iter()
        .filter(|e| e.format.eq("json"))
        .map(|e| {
            let example_value = serde_json::from_str::<serde_json::Value>(e.value.as_str())?;
            Ok((e.name.clone(), example_value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swagger() {
        let codegen = Swagger { skip_failed: true };
        let _ = codegen.generate_for_folder(
            &PathBuf::from("src/codegen/fixtures/specs"),
            &PathBuf::from("src/codegen/fixtures/swagger"),
        );
    }
}
