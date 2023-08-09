use super::*;
use crate::program::expression::convert_expression;
use crate::program::statement_declare_and_store::convert_declare_and_bind;

pub fn convert_statement<'a>(ast: &'a AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    match ast.as_statement().unwrap() {
        Statement::DeclareAndBind {
            ident,
            path,
            expression,
        } => convert_declare_and_bind(ident, path, expression, operations)?,
        Statement::Bind {
            reference,
            expression,
        } => {
            convert_expression(expression, operations)?;

            dbg!(reference);
            todo!()
        }
        Statement::Return { expression: expr } => {
            convert_expression(expr, operations)?;
            operations.push(Op::Return);
        }
        Statement::Expression(exp) => convert_expression(exp, operations)?,
    }
    Ok(())
}
