use super::context::Context;
use super::Codegen;
use std::path::PathBuf;

#[derive(Default)]
pub struct Swagger {}

impl Codegen for Swagger {
    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        // let context = Context::load_from_path()?;
        Ok(())
    }
}
