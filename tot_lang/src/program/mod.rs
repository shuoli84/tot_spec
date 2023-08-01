use crate::ast::{Ast, AstNode, AstNodeKind, Expression, Literal, Statement};
use anyhow::bail;
use serde_json::{Number, Value};
use std::borrow::Cow;
use tot_spec::Type;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum ReferenceOrValue<'a> {
    Reference(&'a str),
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
}

fn convert_ast_to_operations<'a>(
    ast_node: &'a AstNode,
    operations: &mut Vec<Op<'a>>,
) -> anyhow::Result<()> {
    match ast_node.kind() {
        AstNodeKind::Statement(stmt) => {}
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
        Statement::Expression(_) => {}
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
        Expression::Reference(_reference) => {
            todo!()
        }
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
    use crate::program::convert_statement;

    #[test]
    fn test_program_declare_and_assign() {
        let ast = AstNode::parse_statement("let i: i32 = 1;").unwrap();

        let mut operations = vec![];
        convert_statement(&ast, &mut operations).unwrap();

        dbg!(operations);
    }
}
