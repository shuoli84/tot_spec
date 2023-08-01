use crate::ast::{Ast, AstNode, AstNodeKind, Expression, Literal, Statement};
use anyhow::bail;
use serde_json::{Number, Value};
use std::borrow::Cow;
use tot_spec::Type;

#[derive(Debug, Eq, PartialEq)]
pub enum Op<'a> {
    Declare {
        name: &'a str,
        ty: Cow<'a, Type>,
    },
    Store {
        name: &'a str,
    },
    Load(ReferenceOrValue<'a>),
    EnterScope,
    ExitScope,
    Call {
        path: &'a str,
        params: Vec<ReferenceOrValue<'a>>,
    },
}

#[derive(Eq, PartialEq, Debug)]
pub enum ReferenceOrValue<'a> {
    Reference(Cow<'a, str>),
    Value(Value),
}

/// program is compact representation for a tot program. It is generated from ast
pub struct Program<'a> {
    operations: Vec<Op<'a>>,
}

impl<'a> Program<'a> {
    pub fn from_ast(ast: &'a Ast) -> anyhow::Result<Self> {
        let ast_node = ast.node();
        let mut operations: Vec<Op<'a>> = vec![];
        convert_ast_to_operations(ast_node, &mut operations)?;
        Ok(Self { operations })
    }

    pub fn operations(&self) -> &[Op] {
        &self.operations
    }
}

fn convert_ast_to_operations<'a>(
    ast_node: &'a AstNode,
    operations: &mut Vec<Op<'a>>,
) -> anyhow::Result<()> {
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

fn convert_statement<'a>(ast: &'a AstNode, operations: &mut Vec<Op<'a>>) -> anyhow::Result<()> {
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

fn convert_declare_and_bind<'a>(
    ident: &'a AstNode,
    path: &'a AstNode,
    expr: &'a AstNode,
    operations: &mut Vec<Op<'a>>,
) -> anyhow::Result<()> {
    let ident = ident.as_ident().unwrap();

    let type_path = path.as_path().unwrap();
    let ty_ = Type::try_parse(type_path)?;

    operations.push(Op::Declare {
        name: ident,
        ty: Cow::Owned(ty_),
    });

    convert_expression(expr, operations)?;
    operations.push(Op::Store { name: ident });

    Ok(())
}

fn convert_expression<'a>(exp: &'a AstNode, operations: &mut Vec<Op<'a>>) -> anyhow::Result<()> {
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
        Expression::Block(_) => {}
    }
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
                    ty: Cow::Owned(Type::I32),
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
}
