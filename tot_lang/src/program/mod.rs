use crate::ast::{Ast, AstNode, AstNodeKind, Expression, Literal, Statement};
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
    pub fn from_ast(ast: &Ast) -> anyhow::Result<Self> {
        let ast_node = ast.node();
        let mut operations: Vec<Op> = vec![];
        convert_ast_to_operations(ast_node, &mut operations)?;
        Ok(Self { operations })
    }

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

fn convert_ast_to_operations(ast_node: &AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    match ast_node.kind() {
        AstNodeKind::Statement(..) => convert_statement(ast_node, operations)?,
        AstNodeKind::Bind { .. } => {}
        AstNodeKind::Ident { .. } => {}
        AstNodeKind::Path { .. } => {}
        AstNodeKind::Block { .. } => {}
        AstNodeKind::Expression(_) => {}
        AstNodeKind::Literal { .. } => {}
        AstNodeKind::Reference { .. } => {}
        AstNodeKind::FuncDef { .. } => {}
        AstNodeKind::FuncSignature { .. } => {}
        AstNodeKind::FuncParam { .. } => {}
        AstNodeKind::If { .. } => {}
        AstNodeKind::For { .. } => {}
        AstNodeKind::Call { .. } => {}
    }

    Ok(())
}

fn convert_statement<'a>(ast: &'a AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    match ast.as_statement().unwrap() {
        Statement::DeclareAndBind {
            ident,
            path,
            expression,
        } => convert_declare_and_bind(ident, path, expression, operations)?,
        Statement::Bind { .. } => {}
        Statement::Return { .. } => {}
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
                Literal::String => Value::String(literal_value.to_string()),
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
        Expression::Call(_) => {}
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
    use std::borrow::Cow;
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
}
