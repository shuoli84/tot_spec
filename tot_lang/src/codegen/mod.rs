use crate::ast::{AstNode, Expression, Literal, Statement};
use crate::codegen::scope::{CodegenScopes, DeferedScopeLock};
use crate::type_repository::{ModelOrType, TypeRepository};
use crate::CodegenBehavior;
use anyhow::{anyhow, bail};
use std::fmt::Write;
use std::sync::Arc;
use tot_spec::{ModelDef, ModelType, Type};

mod scope;

pub struct Codegen {
    behavior: Box<dyn CodegenBehavior>,
    type_repository: Arc<TypeRepository>,
    scopes: CodegenScopes,
}

impl Codegen {
    /// Create a new codegen instance from behavior
    pub fn new(behavior: Box<dyn CodegenBehavior>, type_repository: Arc<TypeRepository>) -> Self {
        Self {
            behavior,
            type_repository,
            scopes: Default::default(),
        }
    }

    fn enter_scope(&mut self) -> DeferedScopeLock {
        DeferedScopeLock::new(self)
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
        let mut self_ = self.enter_scope();

        let Some((sig, body)) = ast.as_func_def() else {
            bail!("ast is not func_def");
        };

        let (ident, params, ret) = sig.as_func_signature().unwrap();

        let ident = ident.as_ident().unwrap();
        let params = {
            let mut param_code_blocks = vec![];
            for param in params {
                param_code_blocks.push(self_.generate_func_param(param)?);
            }

            param_code_blocks
        };

        let ret_type = if let Some(ret_type) = ret {
            Some(self_.generate_type(ret_type)?)
        } else {
            None
        };

        let body = self_.generate_block(body, true)?;
        let func_signature = self_.behavior.codegen_for_func_signature(
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

        self.scopes.update_reference(
            ident.as_ident().unwrap().to_string(),
            path.as_path().unwrap().to_string(),
        );
        dbg!(&self.scopes);

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
                self.scopes
                    .update_reference(ident.to_string(), path.as_path().unwrap().to_string());

                let ty = self.generate_type(path)?;
                let expr_code = self.generate_expression(expression)?;
                writeln!(result, "let mut {ident}: {ty} = {expr_code};")?;
            }
            Statement::Bind {
                reference,
                expression,
            } => {
                let reference = reference.as_reference().unwrap();
                let expr_code = self.generate_expression(expression)?;

                let reference = reference
                    .into_iter()
                    .map(|r| r.as_ident().unwrap())
                    .collect::<Vec<_>>()
                    .join(".");

                writeln!(result, "{reference} = {expr_code};")?;
            }
            Statement::Return { expression } => {
                let expr_code = self.generate_expression(expression)?;
                let ret_code = self.behavior.codegen_for_return(&expr_code, false)?;
                writeln!(result, "{ret_code}")?;
            }
            Statement::Expression(expr) => {
                let expr_code = self.generate_expression(expr)?;
                result.push_str(&expr_code);
                result.push(';');
            }
        }

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

                let source_var_name = "s";
                writeln!(result, "{{")?;
                writeln!(result, "let {source_var_name} = {expr};")?;

                let source_path = self.type_path_for_reference(&value_l_ref)?.to_string();
                let convert_code =
                    self.generate_convert(&source_path, target_path.as_path().unwrap(), "s")?;
                writeln!(result, "{convert_code}")?;
                writeln!(result, "}}")?;
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

    fn generate_convert(
        &mut self,
        from_type_path: &str,
        to_type_path: &str,
        source_var_name: &str,
    ) -> anyhow::Result<String> {
        let source_type = self
            .type_repository
            .type_for_path(from_type_path)?
            .to_owned();
        let target_type = self.type_repository.type_for_path(to_type_path)?.to_owned();

        match (source_type, target_type) {
            (ModelOrType::Type(source_type), ModelOrType::Type(target_type)) => {
                match (source_type, target_type) {
                    (Type::String, Type::String) => {
                        return Ok(format!("{source_var_name}.clone()"))
                    }
                    (
                        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F64 | Type::Bool,
                        Type::String,
                    ) => return Ok(format!("{source_var_name}.to_string()")),
                    (Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F64, Type::F64) => {
                        return Ok(format!("{source_var_name} as f64"))
                    }
                    (Type::Bool, Type::Bool) => return Ok("=".to_string()),

                    (_, Type::Reference(_)) => {
                        unimplemented!()
                    }
                    (Type::Reference(_), _) => {
                        unimplemented!()
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }
            (
                ModelOrType::ModelType(source_model_type, spec_id),
                ModelOrType::Type(target_type),
            ) => {
                // todo: support more type conversions, e.g: new_type around i64
                if !matches!(target_type, Type::Json) {
                    bail!("Only support convert from UDT to json");
                }

                return Ok(format!("serde_json::to_value(&{source_var_name})?"));
            }
            (ModelOrType::Type(source_type), ModelOrType::ModelType(..)) => {
                match source_type {
                    Type::Json => {
                        // only support convert from json to model
                        return Ok(format!("serde_json::from_value({source_var_name})?"));
                    }
                    _ => {
                        bail!(
                            "convert from {:?} to {:?} not supported",
                            from_type_path,
                            to_type_path
                        );
                    }
                }
            }
            (
                ModelOrType::ModelType(source_model_type, source_spec_id),
                ModelOrType::ModelType(target_model_type, target_spec_id),
            ) => {
                // convert model to model, we only generate convert code if these models are
                // compatible
                let code = self.convert_source_model_to_target_model(
                    &source_model_type.clone(),
                    &target_model_type.clone(),
                    source_var_name,
                    to_type_path,
                )?;
                return Ok(code);
            }
        }
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
        let mut self_ = self.enter_scope();

        let Some((statements, value_expr)) = block.as_block() else {
            bail!("ast is not block");
        };

        let mut result = "".to_string();
        writeln!(result, "{{")?;
        for statement in statements {
            let statement_code = self_.generate_statement(statement)?;
            writeln!(result, "{statement_code}")?;
        }
        if let Some(value_expr) = value_expr {
            let expr_code = self_.generate_expression(value_expr)?;

            // if this block is both func_body and also the expr is its value_expr, then
            // we treat it a little bit different
            let is_func_last_exp = is_func_body;
            let ret_code = if is_func_last_exp {
                self_
                    .behavior
                    .codegen_for_return(&expr_code, is_func_last_exp)?
            } else {
                expr_code
            };
            writeln!(result, "{ret_code}")?;
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
                    Literal::String => "string".into(),
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
            Expression::Reference(reference) => {
                self.type_path_for_reference(reference)?.to_string()
            }
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

    /// get type_path for current reference, the type_path for reference changes from time_to_time
    /// e.g: even in same scope, the same reference can be overwritten
    fn type_path_for_reference(&mut self, reference: &AstNode) -> anyhow::Result<&str> {
        let reference = reference.as_reference().unwrap();
        assert_eq!(reference.len(), 1, "current not support reference to field");
        let local_var_ref = &reference[0].as_ident().unwrap();
        let type_path = self
            .scopes
            .lookup_reference_type(local_var_ref)
            .ok_or_else(|| anyhow!("not able to find reference {local_var_ref:?}"))?;
        Ok(type_path)
    }

    /// recursive check whether source able to convert to target, in the spirit
    /// of convert through json. if source -> json -> target succeeds, then this
    /// function should return true.
    /// source_var_name will be consumed after the convert
    fn convert_source_model_to_target_model(
        &mut self,
        source: &ModelDef,
        target: &ModelDef,
        source_var_name: &str,
        target_type_path: &str,
    ) -> anyhow::Result<String> {
        let target_type = self.behavior.codegen_for_type(target_type_path)?;
        let mut code_blocks = vec![format!("{target_type} {{")];

        match (&source.type_, &target.type_) {
            (ModelType::Const { .. }, _) | (_, ModelType::Const { .. }) => {
                bail!("const in convert is not supported yet")
            }
            (ModelType::Struct(source_st), ModelType::Struct(target_st)) => {
                // todo: support extend
                assert!(source_st.extend.is_none() && target_st.extend.is_none());

                let source_fields = &source_st.fields;
                let target_fields = &target_st.fields;

                let mut field_codes = vec![];

                for field in target_fields {
                    let field_name = field.name.as_str();
                    // for each field, find the same name in source
                    // if not found, then if field is optional, then skip
                    // if found, then generate convert code
                    match source_fields
                        .iter()
                        .filter(|f| f.name.eq(&field.name))
                        .nth(0)
                    {
                        None => {
                            if field.required {
                                bail!("field {} is required, but not found in source", field.name);
                            }

                            field_codes.push(format!("{}: None", field.name));
                        }
                        Some(source_field) => {
                            if source_field.type_.inner().eq(&field.type_.inner()) {
                                field_codes.push(format!(
                                    "{field_name}: {source_var_name}.{field_name}.clone()"
                                ))
                            } else {
                                todo!()
                            }
                        }
                    }
                }

                code_blocks.push(field_codes.join(","));
            }
            _ => {
                todo!()
            }
        }

        code_blocks.push("}".to_string());

        Ok(code_blocks.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tot_spec::Context;

    #[derive(Debug, Default)]
    struct TestBehavior {}

    #[async_trait::async_trait]
    impl CodegenBehavior for TestBehavior {
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
                "string" => "String".to_string(),
                "json" => "serde_json::Value".to_string(),
                "i8"
                | "i16"
                | "i32"
                | "i64"
                | "f64"
                | "bool"
                | "base::FirstRequest"
                | "base::FirstResponse"
                | "base::SecondRequest"
                | "base::SecondResponse"
                | "spec::NewTypeStruct"
                | "spec::TestStruct" => path.to_string(),
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
        let folder = "src/codegen/fixtures";
        for case_name in ["simple", "type_conversion", "assign_to_field"] {
            println!("* testing {case_name}");
            let tot_file = format!("{folder}/{case_name}.tot");
            let rs_file = format!("{folder}/{case_name}.rs");

            let code = std::fs::read_to_string(tot_file).unwrap();
            let mut codegen = Codegen::new(
                Box::new(TestBehavior::default()),
                Arc::new(TypeRepository::new(
                    Context::new_from_folder(&PathBuf::from(folder)).unwrap(),
                )),
            );
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
