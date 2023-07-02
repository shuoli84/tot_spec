use std::path::PathBuf;

pub mod java_jackson;
pub mod py_dataclass;
pub mod rs_serde;
pub mod swagger;
pub mod swift_codable;
pub mod utils;

pub mod spec_folder;

pub trait Codegen {
    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()>;
}
