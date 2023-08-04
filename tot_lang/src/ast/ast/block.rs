use super::*;
use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::statement::parse_statement;
use anyhow::{bail};
use pest::iterators::Pair;

pub fn parse_block(pair: Pair<Rule>) -> anyhow::Result<AstNode> {
    assert!(matches!(pair.as_rule(), Rule::block));

    let span = pair.as_span().into();
    let inner = pair.into_inner();

    let mut statements = vec![];
    let mut value_expr = None;

    for inner in inner {
        match inner.as_rule() {
            Rule::statement => {
                statements.push(parse_statement(inner)?);
            }
            Rule::block_value => {
                value_expr = Some(Box::new(parse_expression(try_take_first(
                    inner.into_inner(),
                )?)?));
            }
            _ => {
                bail!("ast node not supported {inner:?}")
            }
        }
    }

    Ok(AstNode {
        kind: AstNodeKind::Block {
            statements,
            value_expr,
        },
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_parse_block_empty() {
        let parsed = GrammarParser::parse(Rule::block, "{}")
            .unwrap()
            .nth(0)
            .unwrap();
        let block = parse_block(parsed).unwrap();
        let (stmts, value_expr) = block.as_block().unwrap();
        assert!(stmts.is_empty());
        assert!(!value_expr.is_some());
    }

    #[test]
    fn test_parse_block_expr() {
        let parsed = GrammarParser::parse(Rule::block, "{ 1 }")
            .unwrap()
            .nth(0)
            .unwrap();
        let block = parse_block(parsed).unwrap();
        let (stmts, value_expr) = block.as_block().unwrap();
        assert!(stmts.is_empty());
        assert!(value_expr.is_some());
    }

    #[test]
    fn test_parse_block_complex() {
        let parsed = GrammarParser::parse(
            Rule::block,
            r#"{ 
                let a: i32 = 1; 
                let _b: i32 = 2; 
                a
            }"#,
        )
        .unwrap()
        .nth(0)
        .unwrap();
        let block = parse_block(parsed).unwrap();
        let (stmts, value_expr) = block.as_block().unwrap();
        assert_eq!(stmts.len(), 2);
        assert!(value_expr.is_some());
    }
}
