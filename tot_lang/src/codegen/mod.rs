use crate::ast::{AstNode, Expression, Literal, Statement};
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

        let (ident, params, ret) = sig.as_func_signature().unwrap();

        let ident = ident.as_ident().unwrap();
        let params = {
            let mut param_code_blocks = vec![];
            for param in params {
                param_code_blocks.push(self.generate_func_param(param)?);
            }

            param_code_blocks
        };

        let ret_type = if let Some(ret_type) = ret {
            Some(self.generate_type(ret_type)?)
        } else {
            None
        };

        let body = self.generate_block(body, true)?;
        let func_signature = self.behavior.codegen_for_func_signature(
            ident,
            &params,
            ret_type.as_ref().map(|s| s.as_str()),
        )?;

        let mut result = func_signature;
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
                let ty = self.generate_type(path)?;
                let expr_code = self.generate_expression(expression)?;
                writeln!(result, "let mut {ident}: {ty} = {expr_code};")?;
            }
            Statement::Bind { ident, expression } => {
                let ident = ident.as_ident().unwrap();
                let expr_code = self.generate_expression(expression)?;
                writeln!(result, "{ident} = {expr_code};")?;
            }
            Statement::Return { expression } => {
                let expr_code = self.generate_expression(expression)?;
                let ret_code = self.behavior.codegen_for_return(&expr_code, false)?;
                writeln!(result, "{ret_code}")?;
            }
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
            Expression::Convert(c) => {
                let (value_l_ref, target_path) = c.as_convert().unwrap();
                let expr = self.generate_reference(value_l_ref)?;

                writeln!(result, "{expr};")?;

                let target_type = self.generate_type(target_path)?;
                let expr_type = self.type_path_for_reference(&value_l_ref)?;

                writeln!(result, "// todo: {expr_type} -> {target_type}")?;
            }
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
                let code = self.generate_reference(reference)?;
                write!(result, "{code}")?;
            }
            Expression::Call(call) => {
                let call_code = self.generate_call(call)?;
                writeln!(result, "{}", call_code)?;
            }
            Expression::If(if_exp) => {
                let if_code = self.generate_if(if_exp)?;
                writeln!(result, "{if_code}")?;
            }
            Expression::Block(block) => {
                let block_code = self.generate_block(block, false)?;
                writeln!(result, "{block_code}")?;
            }
        }

        Ok(result.trim().to_string())
    }

    fn generate_reference(&mut self, ast: &AstNode) -> anyhow::Result<String> {
        let reference_components = ast.as_reference().unwrap();
        let mut result = "".to_string();
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
        Ok(result)
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

        let block_code = self.generate_block(block, false)?;
        result.push_str(&block_code);

        if let Some(else_block) = else_block {
            result.push_str(" else ");
            let code = self.generate_block(else_block, false)?;
            result.push_str(&code);
        }

        Ok(result)
    }

    fn generate_block(&mut self, block: &AstNode, is_func_body: bool) -> anyhow::Result<String> {
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

            // if this block is both func_body and also the expr is its value_expr, then
            // we treat it a little bit different
            let is_func_last_exp = is_func_body;

            let ret_code = self
                .behavior
                .codegen_for_return(&expr_code, is_func_last_exp)?;
            writeln!(result, "{ret_code}")?
        }
        writeln!(result, "}}")?;

        Ok(result)
    }

    /// get the type for expr
    fn type_path_for_expr(&mut self, expr: &AstNode) -> anyhow::Result<String> {
        let Some(expr) = expr.as_expression() else {
            bail!("node is not expression: {expr:?}");
        };

        Ok(match expr {
            Expression::Literal(l) => {
                let (literal_type, literal_value) = l.as_literal().unwrap();
                match literal_type {
                    Literal::String => "String".into(),
                    Literal::Number => if literal_value.parse::<f64>().is_ok() {
                        "f64"
                    } else if literal_value.parse::<i64>().is_ok() {
                        "i64"
                    } else {
                        "u64"
                    }
                    .into(),
                    Literal::Boolean => "bool".into(),
                }
            }
            Expression::Reference(reference) => self.type_path_for_reference(reference)?,
            Expression::Call(call_node) => {
                let (path, _) = call_node.as_call().unwrap();
                let func_path = path.as_path().unwrap();
                self.behavior.return_type_for_method(func_path)?
            }
            Expression::If(if_node) => {
                let (_, block, _) = if_node.as_if().unwrap();

                let (_, value_expr) = block.as_block().unwrap();
                match value_expr {
                    None => "()".to_string(),
                    Some(expr) => self.type_path_for_expr(expr)?,
                }
            }
            Expression::Block(block) => {
                let (_, value_expr) = block.as_block().unwrap();
                match value_expr {
                    None => "()".to_string(),
                    Some(expr) => self.type_path_for_expr(expr)?,
                }
            }
            Expression::Convert(convert) => {
                let (_, target_path) = convert.as_convert().unwrap();
                target_path.as_path().unwrap().to_string()
            }
        })
    }

    fn type_path_for_reference(&mut self, _reference: &AstNode) -> anyhow::Result<String> {
        // let reference = reference.as_reference().unwrap();
        // we need to keep track of type info for all reachable references
        Ok("todo(reference)".into())
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
        async fn runtime_call_method(
            &mut self,
            _method: &str,
            _params: &[Value],
        ) -> anyhow::Result<Value> {
            todo!()
        }

        fn return_type_for_method(&mut self, name: &str) -> anyhow::Result<String> {
            match name {
                _ => {
                    bail!("{name} not supported yet");
                }
            }
        }

        fn codegen_for_func_signature(
            &mut self,
            name: &str,
            params: &[String],
            ret_type: Option<&str>,
        ) -> anyhow::Result<String> {
            // generate async functions
            let params = params.join(",");
            Ok(if let Some(ret_type) = ret_type {
                format!("async fn {name}({params}) -> anyhow::Result<{ret_type}>")
            } else {
                format!("async fn {name}({params} -> anyhow::Result<()>")
            })
        }

        fn codegen_for_type(&mut self, path: &str) -> anyhow::Result<String> {
            Ok(match path {
                "String" => "String".to_string(),
                "json" => "serde_json::Value".to_string(),
                "FirstRequest" => "serde_json::Value".to_string(),
                "SecondResponse" => "serde_json::Value".to_string(),
                _ => {
                    bail!("type {path} not supported")
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
                    format!("my_crate::a::b::sync_func({params_code})?")
                }
                "a::b::async_func" => {
                    format!("my_crate::a::b::async_func({params_code}).await?")
                }
                "first_method" => {
                    format!("my_crate::a::b::first({params_code}).await?")
                }
                "second_method" => {
                    format!("my_crate::a::b::second({params_code}).await?")
                }
                _ => {
                    bail!("call \"{path}\" not supported")
                }
            })
        }

        fn codegen_for_return(&mut self, expr: &str, is_last: bool) -> anyhow::Result<String> {
            if is_last {
                Ok(format!("Ok({expr})"))
            } else {
                Ok(format!("return Ok({expr});"))
            }
        }
    }

    #[test]
    fn test_codegen() {
        for (tot_file, rs_file) in [
            (
                "src/codegen/fixtures/simple.tot",
                "src/codegen/fixtures/simple.rs",
            ),
            (
                "src/codegen/fixtures/type_conversion.tot",
                "src/codegen/fixtures/type_conversion.rs",
            ),
        ] {
            let code = std::fs::read_to_string(tot_file).unwrap();
            let mut codegen = Codegen::new(Box::new(TestBehavior::default()));
            let ast = AstNode::parse_file(&code).unwrap();

            let code = codegen.generate_file(&ast).unwrap();
            println!("{}", code);
            let code_ast = syn::parse_file(&code).unwrap();
            let formatted_gen_code = prettyplease::unparse(&code_ast);

            let expected_rs_code = std::fs::read_to_string(rs_file).unwrap();
            let code_ast = syn::parse_file(&expected_rs_code).unwrap();
            let formatted_expect_code = prettyplease::unparse(&code_ast);

            pretty_assertions::assert_eq!(formatted_gen_code.trim(), formatted_expect_code.trim());
        }
    }
}
