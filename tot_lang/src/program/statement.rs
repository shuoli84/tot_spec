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

            let reference_components = reference.as_reference().unwrap();
            match reference_components.split_first() {
                None => {
                    bail!("reference is empty");
                }
                Some((var_name, field_path)) => {
                    operations.push(Op::Store {
                        name: var_name.as_ident().unwrap().to_string(),
                        path: field_path
                            .into_iter()
                            .map(|f| f.as_ident().unwrap().to_string())
                            .collect(),
                    });
                }
            }
        }
        Statement::Return { expression: expr } => {
            convert_expression(expr, operations)?;
            operations.push(Op::Return);
        }
        Statement::Expression(exp) => convert_expression(exp, operations)?,
    }
    Ok(())
}
