use super::frame::Frame;
use crate::ast::Ast;

/// The virtual machine, which stores all runtime state for one run
#[derive(Default, Debug)]
pub struct Vm {
    frame: Frame,
}

impl Vm {
    /// Execute the ast
    pub async fn execute(&mut self, ast: &Ast) -> anyhow::Result<()> {
        todo!()
    }
}
