use super::Codegen;
use crate::codegen::context::Context;
use std::path::PathBuf;

#[derive(Default)]
pub struct Swagger {}

impl Codegen for Swagger {
    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        let _context = Context::new_from_folder(folder)?;

        Ok(())
    }
}
