mod ast;
pub use ast::{AstNode, AstNodeKind, Expression, Literal, Statement};

mod grammar;
pub mod instruction;

/// The ast for lang
pub struct Ast {
    node: AstNode,
}

impl Ast {
    pub fn node(&self) -> &AstNode {
        &self.node
    }
}
