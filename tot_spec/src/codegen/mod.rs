use std::path::PathBuf;
use crate::{Context, Definition};
use crate::codegen::spec_folder::Entry;

pub mod java_jackson;
pub mod py_dataclass;
pub mod rs_serde;
pub mod swift_codable;
pub mod utils;

pub mod spec_folder;

pub trait Codegen {
    fn generate_for_folder(&self, folder: &PathBuf, codegen: &str, output: &PathBuf) -> anyhow::Result<()>;
}
