use crate::ast::{AstNode, Expression, Literal, Statement};
use anyhow::bail;
use serde_json::{Number, Value};
use tot_spec::Type;

#[derive(Debug, Eq, PartialEq)]
pub enum Op {
    Declare {
        name: String,
        ty: Type,
    },
    Store {
        name: String,
    },
    /// Load the value to register
    Load(ReferenceOrValue),
    EnterScope,
    ExitScope,
    Call {
        path: String,
        params: Vec<ReferenceOrValue>,
    },
    Return,
}

#[derive(Eq, PartialEq, Debug)]
pub enum ReferenceOrValue {
    Reference(String),
    Value(Value),
}

/// program is compact representation for a tot program. It is generated from ast
pub struct Program {
    operations: Vec<Op>,
}

impl Program {
    pub fn from_statement(code: &str) -> anyhow::Result<Self> {
        let ast = AstNode::parse_statement(code.trim())?;
        let mut operations = vec![];
        convert_statement(&ast, &mut operations)?;
        Ok(Self { operations })
    }

    pub fn operations(&self) -> &[Op] {
        &self.operations
    }
}

fn convert_statement<'a>(ast: &'a AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    match ast.as_statement().unwrap() {
        Statement::DeclareAndBind {
            ident,
            path,
            expression,
        } => convert_declare_and_bind(ident, path, expression, operations)?,
        Statement::Bind { .. } => {}
        Statement::Return { expression: expr } => {
            convert_expression(expr, operations)?;
            // with the expr's value stored at register. Now return
            operations.push(Op::Return);
        }
        Statement::Expression(exp) => convert_expression(exp, operations)?,
    }
    Ok(())
}

fn convert_declare_and_bind(
    ident: &AstNode,
    path: &AstNode,
    expr: &AstNode,
    operations: &mut Vec<Op>,
) -> anyhow::Result<()> {
    let ident = ident.as_ident().unwrap().to_string();

    let type_path = path.as_path().unwrap();
    let ty = Type::try_parse(type_path)?;

    operations.push(Op::Declare {
        name: ident.clone(),
        ty: ty,
    });

    convert_expression(expr, operations)?;
    operations.push(Op::Store { name: ident });

    Ok(())
}

fn convert_expression(exp: &AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    let exp = exp.as_expression().unwrap();
    match exp {
        Expression::Literal(literal_node) => {
            let (literal_type, literal_value) = literal_node.as_literal().unwrap();
            let value = match literal_type {
                Literal::String => Value::String(snailquote::unescape(literal_value)?),
                Literal::Number => Value::Number(Number::from(literal_value.parse::<i64>()?)),
                Literal::Boolean => Value::Bool(match literal_value {
                    "true" => true,
                    "false" => false,
                    _ => bail!("invalid bool literal, {literal_value}"),
                }),
            };
            operations.push(Op::Load(ReferenceOrValue::Value(value)));
        }
        Expression::Reference(reference) => operations.push(Op::Load(ReferenceOrValue::Reference(
            reference
                .as_reference()
                .unwrap()
                .iter()
                .map(|i| i.as_ident().unwrap())
                .collect::<Vec<_>>()
                .join(".")
                .into(),
        ))),
        Expression::Call(call) => {
            // each call creates a new scope
            operations.push(Op::EnterScope);
            let (path, params) = call.as_call().unwrap();

            let mut param_references: Vec<String> = vec![];

            for (idx, param) in params.iter().enumerate() {
                convert_expression(param, operations)?;

                let param_name = format!("_{idx}");
                operations.push(Op::Declare {
                    name: param_name.clone(),
                    ty: Type::Json,
                });

                param_references.push(param_name.clone());
                operations.push(Op::Store { name: param_name });
            }

            operations.push(Op::Call {
                path: path.as_path().unwrap().to_string(),
                params: param_references
                    .into_iter()
                    .map(|p| ReferenceOrValue::Reference(p))
                    .collect(),
            });

            operations.push(Op::ExitScope);
        }
        Expression::If(_) => {}
        Expression::For(_) => {}
        Expression::Block(block) => {
            convert_block(block, operations)?;
        }
    }
    Ok(())
}

fn convert_block(block: &AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    let Some((statements, value_expr)) = block.as_block() else {
        bail!("node is block");
    };

    operations.push(Op::EnterScope);

    for statement in statements {
        convert_statement(statement, operations)?;
    }
    if let Some(value_expr) = value_expr {
        convert_expression(value_expr, operations)?;
    }

    operations.push(Op::ExitScope);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ast::AstNode;
    use crate::program::{convert_statement, Op, ReferenceOrValue};
    use serde_json::{Number, Value};
    use tot_spec::Type;

    #[test]
    fn test_program_declare_and_assign() {
        let ast = AstNode::parse_statement("let i: i32 = 1;").unwrap();

        let mut operations = vec![];
        convert_statement(&ast, &mut operations).unwrap();

        assert_eq!(
            operations,
            vec![
                Op::Declare {
                    name: "i".into(),
                    ty: Type::I32,
                },
                Op::Load(ReferenceOrValue::Value(Value::Number(Number::from(1)))),
                Op::Store { name: "i".into() }
            ]
        )
    }

    #[test]
    fn test_program_load_literal() {
        let ast = AstNode::parse_statement("\"hello\";").unwrap();

        let mut operations = vec![];
        convert_statement(&ast, &mut operations).unwrap();

        assert_eq!(
            operations,
            vec![Op::Load(ReferenceOrValue::Value("hello".into()))]
        );
    }

    #[test]
    fn test_program_load_reference() {
        let ast = AstNode::parse_statement("i;").unwrap();

        let mut operations = vec![];
        convert_statement(&ast, &mut operations).unwrap();

        assert_eq!(
            operations,
            vec![Op::Load(ReferenceOrValue::Reference("i".into()))]
        );
    }

    #[test]
    fn test_program_block() {
        let ast = AstNode::parse_statement(
            r#"{
            let i: i32 = 1;
            let j: i64 = 2;
            let k: i32 = {
                100
            };
            {
                // do nothing
            };
            i
        };"#,
        )
        .unwrap();

        let mut operations = vec![];
        assert!(convert_statement(&ast, &mut operations).is_ok());
    }

    #[test]
    fn test_program_call() {
        let ast = AstNode::parse_statement(r#"a(1, 2, 3);"#).unwrap();

        let mut operations = vec![];
        assert!(convert_statement(&ast, &mut operations).is_ok());
        dbg!(operations);
    }
}
