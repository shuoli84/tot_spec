use crate::ast::{Ast, AstNode, Expression, Literal, Statement};
use crate::Behavior;
use anyhow::{anyhow, bail};
use std::fmt::Write;

pub struct Codegen {
    behavior: Box<dyn Behavior>,
}

impl Codegen {
    /// generate rust code for program
    pub fn generate(&self, ast: &Ast) -> anyhow::Result<String> {
        let mut result = "".to_string();
        let ast_node = ast.node();

        Ok(result)
    }
}

fn generate_file(ast: &AstNode) -> anyhow::Result<String> {
    let Some((func_defs)) = ast.as_file() else {
        bail!("ast is not file");
    };

    let mut result = "".to_string();

    for func_def in func_defs {
        let func_code = generate_func_def(func_def)?;
        writeln!(result, "{}", func_code)?;
    }

    Ok(result)
}

fn generate_func_def(ast: &AstNode) -> anyhow::Result<String> {
    let Some((sig, body)) = ast.as_func_def() else  {
        bail!("ast is not func_def");
    };

    let mut result = "".to_string();

    let (ident, params, ret) = sig.as_func_signature().unwrap();

    let ident = ident.as_ident().unwrap();
    let params = "";
    let return_type = "String";
    writeln!(result, "fn {ident}({params}) -> {return_type}")?;

    let body = generate_block(body)?;
    result.push_str(&body);

    Ok(result)
}

fn generate_statement(ast: &AstNode) -> anyhow::Result<String> {
    let statement = ast
        .as_statement()
        .ok_or_else(|| anyhow!("ast node is not statement"))?;

    let mut result = "".to_string();

    match statement {
        Statement::DeclareAndBind {
            ident,
            path,
            expression,
        } => {
            let ident = ident.as_ident().unwrap();
            // todo: fix this
            let ty = "String";
            let expr_code = generate_expression(expression)?;
            writeln!(result, "let mut {ident}: {ty} = {expr_code};")?;
        }
        Statement::Bind { ident, expression } => {
            let ident = ident.as_ident().unwrap();
            let expr_code = generate_expression(expression)?;
            writeln!(result, "{ident} = {expr_code};")?;
        }
        Statement::Return { .. } => {}
        Statement::Expression(expr) => {
            let expr_code = generate_expression(expr)?;
            result.push_str(&expr_code);
        }
    }

    Ok(result)
}

fn generate_expression(ast: &AstNode) -> anyhow::Result<String> {
    let Some(expression) = ast.as_expression() else {
        bail!("ast node is not expression");
    };

    let mut result = "".to_string();

    match expression {
        Expression::Literal(l) => {
            let (literal_type, literal_value) = l.as_literal().unwrap();
            match literal_type {
                Literal::String => {
                    writeln!(result, "{literal_value}.to_string()")?;
                }
                Literal::Number => {
                    writeln!(result, "{literal_value}")?;
                }
                Literal::Boolean => {
                    writeln!(result, "{literal_value}")?;
                }
            }
        }
        Expression::Reference(reference) => {
            let reference_components = reference.as_reference().unwrap();
            for (idx, component) in reference_components.iter().enumerate() {
                let component_name = component.as_ident().unwrap();
                if idx == 0 {
                    write!(result, "{component_name}")?;
                } else {
                    write!(result, ".{component_name}")?;
                }

                // force a clone as no reference are generated for now
                write!(result, ".clone()")?;
            }
        }
        Expression::Call(_) => {}
        Expression::If(if_exp) => {
            let if_code = generate_if(if_exp)?;
            writeln!(result, "{if_code}")?;
        }
        Expression::For(_) => {}
        Expression::Block(block) => {
            let block_code = generate_block(block)?;
            writeln!(result, "{block_code}")?;
        }
    }

    Ok(result.trim().to_string())
}

fn generate_if(ast: &AstNode) -> anyhow::Result<String> {
    let Some((condition, block, else_block)) = ast.as_if() else {
        bail!("ast is not if_exp");
    };

    let mut result = "".to_string();

    result.push_str("if ");

    let condition_code = generate_expression(condition)?;
    result.push_str(&condition_code);

    let block_code = generate_block(block)?;
    result.push_str(&block_code);

    if let Some(else_block) = else_block {
        result.push_str(" else ");
        let code = generate_block(else_block)?;
        result.push_str(&code);
    }

    Ok(result)
}

fn generate_block(block: &AstNode) -> anyhow::Result<String> {
    let Some((statements, value_expr)) = block.as_block() else {
        bail!("ast is not block");
    };

    let mut result = "".to_string();
    writeln!(result, "{{")?;
    for statement in statements {
        let statement_code = generate_statement(statement)?;
        writeln!(result, "{statement_code}")?;
    }
    if let Some(value_expr) = value_expr {
        let expr_code = generate_expression(value_expr)?;
        writeln!(result, "{expr_code}")?
    }
    writeln!(result, "}}")?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen() {
        let ast = AstNode::parse_file(
            r#"fn hello() -> String {
                let i: String = "hello";
                i = "world";
                let j: String = i;
                let k: String = {
                    if true {
                        "foo"
                    } else {
                        "bar"
                    }
                };
                k
        }"#,
        )
        .unwrap();
        let code = generate_file(&ast).unwrap();

        println!("{code}");

        let code_ast = syn::parse_file(&code).unwrap();
        let formatted_code = prettyplease::unparse(&code_ast);
        println!("{}", formatted_code);
    }
}
