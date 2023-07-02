use super::Codegen;
use std::path::PathBuf;

#[derive(Default)]
pub struct Swagger {}

impl Codegen for Swagger {
    fn generate_for_folder(&self, _folder: &PathBuf, _output: &PathBuf) -> anyhow::Result<()> {
        todo!()
    }
}
