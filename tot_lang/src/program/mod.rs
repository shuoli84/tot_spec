use crate::ast::{AstNode, Expression, Literal, Statement};
use crate::program::block::convert_block;
use crate::program::expression::convert_expression;
use crate::program::statement::convert_statement;
use anyhow::{anyhow, bail};
use serde_json::{Number, Value};
use std::borrow::Cow;
use tot_spec::Type;

mod block;
mod expression;
mod expression_if;
mod statement;
mod statement_declare_and_store;

#[derive(Debug, Eq, PartialEq)]
pub enum Op {
    Declare {
        name: String,
        type_path: Cow<'static, str>,
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
    Convert {
        target_path: String,
    },
    /// Return from current execution program
    Return,
    /// Skip the next n ops if current register value evaluates to false, it must be in same scope, otherwise
    /// you may load/save value from/to wrong scope
    /// Used to implement if expression
    JumpIfFalse(usize),
    /// Skip the next n ops
    Jump(usize),
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
                    type_path: "i32".into(),
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
    fn test_program_call() {
        let ast = AstNode::parse_statement(r#"a(1, 2, 3);"#).unwrap();

        let mut operations = vec![];
        assert!(convert_statement(&ast, &mut operations).is_ok());
        dbg!(operations);
    }

    #[test]
    fn test_program_convert() {
        let ast = AstNode::parse_statement(r#"i as Request;"#).unwrap();

        let mut operations = vec![];
        assert!(convert_statement(&ast, &mut operations).is_ok());
        dbg!(operations);
    }
}
