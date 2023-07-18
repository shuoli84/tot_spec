use std::path::PathBuf;

mod context;
pub mod java_jackson;
pub mod py_dataclass;
pub mod rs_serde;
pub mod style;
pub mod swagger;
pub mod swift_codable;
pub mod utils;

pub trait Codegen {
    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()>;
}
