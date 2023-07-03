use super::Codegen;
use crate::codegen::context::Context;
use openapi::{Info, Spec};
use std::path::PathBuf;

#[derive(Default)]
pub struct Swagger {}

impl Codegen for Swagger {
    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        let context = Context::new_from_folder(folder)?;

        let openapi_spec = Spec {
            swagger: "".to_string(),
            info: Info {
                title: "DCU swagger".to_string(),
                version: "0.1.0".to_string(),
                terms_of_service: None,
            },
            paths: Default::default(),
            definitions: Default::default(),
            schemes: None,
            host: None,
            base_path: None,
            consumes: None,
            produces: None,
            parameters: None,
            responses: None,
            security_definitions: None,
            tags: None,
        };

        for (spec, def) in context.iter_specs() {
            dbg!(spec);
        }

        let output_file = output.join("swagger.json");
        let json_str = serde_json::to_string_pretty(&openapi_spec)?;

        std::fs::write(&output_file, json_str)?;

        Ok(())
    }
}
