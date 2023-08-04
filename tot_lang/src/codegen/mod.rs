use crate::ast::{Ast, AstNode, Expression, Literal, Statement};
use crate::Behavior;
use anyhow::{anyhow, bail};
use std::fmt::Write;

pub struct Codegen {
    behavior: Box<dyn Behavior>,
}

impl Codegen {
    /// Create a new codegen instance from behavior
    pub fn new(behavior: Box<dyn Behavior>) -> Self {
        Self { behavior }
    }

    pub fn generate_file(&mut self, ast: &AstNode) -> anyhow::Result<String> {
        let Some(func_defs) = ast.as_file() else {
            bail!("ast is not file");
        };

        let mut result = "".to_string();

        for func_def in func_defs {
            let func_code = self.generate_func_def(func_def)?;
            writeln!(result, "{}", func_code)?;
        }

        Ok(result)
    }

    fn generate_func_def(&mut self, ast: &AstNode) -> anyhow::Result<String> {
        let Some((sig, body)) = ast.as_func_def() else {
            bail!("ast is not func_def");
        };

        let mut result = "".to_string();

        let (ident, params, ret) = sig.as_func_signature().unwrap();

        let ident = ident.as_ident().unwrap();
        let params = {
            let mut param_code_blocks = vec![];
            for param in params {
                param_code_blocks.push(self.generate_func_param(param)?);
            }

            param_code_blocks.join(", ")
        };

        let return_type = "String";
        writeln!(result, "fn {ident}({params}) -> {return_type}")?;

        let body = self.generate_block(body)?;
        result.push_str(&body);

        Ok(result)
    }

    fn generate_func_param(&mut self, param: &AstNode) -> anyhow::Result<String> {
        let Some((ident, path)) = param.as_func_param() else {
            bail!("ast node is not func param");
        };

        let mut result = "".to_string();

        let rs_type = self.generate_type(path)?;

        writeln!(result, "{}: {}", ident.as_ident().unwrap(), rs_type)?;

        Ok(result)
    }

    fn generate_type(&mut self, path: &AstNode) -> anyhow::Result<String> {
        let Some(path) = path.as_path() else {
            bail!("ast node is not path");
        };
        self.behavior.codegen_for_type(path)
    }

    fn generate_statement(&mut self, ast: &AstNode) -> anyhow::Result<String> {
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
                let expr_code = self.generate_expression(expression)?;
                writeln!(result, "let mut {ident}: {ty} = {expr_code};")?;
            }
            Statement::Bind { ident, expression } => {
                let ident = ident.as_ident().unwrap();
                let expr_code = self.generate_expression(expression)?;
                writeln!(result, "{ident} = {expr_code};")?;
            }
            Statement::Return { .. } => {}
            Statement::Expression(expr) => {
                let expr_code = self.generate_expression(expr)?;
                result.push_str(&expr_code);
            }
        }

        // statement ends with ;
        result.push(';');

        Ok(result)
    }

    fn generate_expression(&mut self, ast: &AstNode) -> anyhow::Result<String> {
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
            Expression::Call(call) => {
                let call_code = self.generate_call(call)?;
                writeln!(result, "{}", call_code)?;
            }
            Expression::If(if_exp) => {
                let if_code = self.generate_if(if_exp)?;
                writeln!(result, "{if_code}")?;
            }
            Expression::For(_) => {
                todo!()
            }
            Expression::Block(block) => {
                let block_code = self.generate_block(block)?;
                writeln!(result, "{block_code}")?;
            }
        }

        Ok(result.trim().to_string())
    }

    fn generate_call(&mut self, ast: &AstNode) -> anyhow::Result<String> {
        let Some((path, params)) = ast.as_call() else {
            bail!("ast is not call");
        };

        let mut params_code = vec![];
        for param in params {
            params_code.push(self.generate_expression(param)?);
        }

        let call_code = self
            .behavior
            .codegen_for_call(path.as_path().unwrap(), &params_code)?;

        Ok(call_code)
    }

    fn generate_if(&mut self, ast: &AstNode) -> anyhow::Result<String> {
        let Some((condition, block, else_block)) = ast.as_if() else {
            bail!("ast is not if_exp");
        };

        let mut result = "".to_string();

        result.push_str("if ");

        let condition_code = self.generate_expression(condition)?;
        result.push_str(&condition_code);

        let block_code = self.generate_block(block)?;
        result.push_str(&block_code);

        if let Some(else_block) = else_block {
            result.push_str(" else ");
            let code = self.generate_block(else_block)?;
            result.push_str(&code);
        }

        Ok(result)
    }

    fn generate_block(&mut self, block: &AstNode) -> anyhow::Result<String> {
        let Some((statements, value_expr)) = block.as_block() else {
            bail!("ast is not block");
        };

        let mut result = "".to_string();
        writeln!(result, "{{")?;
        for statement in statements {
            let statement_code = self.generate_statement(statement)?;
            writeln!(result, "{statement_code}")?;
        }
        if let Some(value_expr) = value_expr {
            let expr_code = self.generate_expression(value_expr)?;
            writeln!(result, "{expr_code}")?
        }
        writeln!(result, "}}")?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[derive(Debug, Default)]
    struct TestBehavior {}

    #[async_trait::async_trait]
    impl Behavior for TestBehavior {
        async fn execute(&mut self, method: &str, params: &[Value]) -> anyhow::Result<Value> {
            todo!()
        }

        fn codegen_for_type(&mut self, path: &str) -> anyhow::Result<String> {
            Ok(match path {
                "String" => "String".to_string(),
                _ => {
                    todo!()
                }
            })
        }

        fn codegen_for_call(&mut self, path: &str, params: &[String]) -> anyhow::Result<String> {
            let params_code = params.join(",");

            Ok(match path {
                "print" => {
                    let params_count = params.len();
                    let place_holder = "{}".repeat(params_count);
                    format!("println!(\"{place_holder}\", {params_code})")
                }
                "a::b::sync_func" => {
                    format!("my_crate::a::b::sync_func({params_code})")
                }
                "a::b::async_func" => {
                    format!("my_crate::a::b::async_func({params_code}).await")
                }
                _ => {
                    bail!("call \"{path}\" not supported")
                }
            })
        }
    }

    #[test]
    fn test_codegen() {
        let mut codegen = Codegen::new(Box::new(TestBehavior::default()));
        let ast = AstNode::parse_file(
            r#"fn hello(i: String) -> String {
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
                print(k);
                let sync_call_result: String = a::b::sync_func(k);
                let sync_call_result: String = a::b::sync_func(k);
                let async_call_result: String = a::b::async_func(k);
                k
            }"#,
        )
        .unwrap();

        dbg!(&ast);

        let code = codegen.generate_file(&ast).unwrap();

        println!("{code}");

        let code_ast = syn::parse_file(&code).unwrap();
        let formatted_code = prettyplease::unparse(&code_ast);
        println!("{}", formatted_code);
    }
}
