use std::path::PathBuf;

pub mod context;
pub mod java_jackson;
pub mod py_dataclass;
pub mod rs_serde;
pub mod style;
pub mod swagger;
pub mod swift_codable;
pub mod utils;

pub trait Codegen {
    fn load_from_folder(folder: &PathBuf) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn generate_for_folder(&self, folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()>;
}
