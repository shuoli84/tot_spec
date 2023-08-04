use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::ident::parse_ident;
use crate::ast::ast::path::parse_path;
use crate::ast::ast::{try_next, try_take_first, AstNode, AstNodeKind, Statement};
use crate::ast::grammar::Rule;
use anyhow::bail;
use pest::iterators::Pair;

pub fn parse_statement(pair: Pair<Rule>) -> anyhow::Result<AstNode> {
    assert!(matches!(pair.as_rule(), Rule::statement));

    let span = pair.as_span().into();
    let inner = try_take_first(pair.into_inner())?;

    let inner = match inner.as_rule() {
        Rule::expression => Statement::Expression(Box::new(parse_expression(inner)?)),
        Rule::declare_and_bind => {
            let mut components = inner.into_inner();
            let ident = try_next(&mut components)?;
            let ident = parse_ident(ident);
            let path = try_next(&mut components)?;
            let path = parse_path(path);
            let expression = try_next(&mut components)?;
            let expr = parse_expression(expression)?;

            Statement::DeclareAndBind {
                ident: Box::new(ident),
                path: Box::new(path),
                expression: Box::new(expr),
            }
        }
        Rule::bind => {
            let mut components = inner.into_inner();
            let ident = try_next(&mut components)?;
            let ident = parse_ident(ident);
            let expression = try_next(&mut components)?;
            let expr = parse_expression(expression)?;

            Statement::Bind {
                ident: Box::new(ident),
                expression: Box::new(expr),
            }
        }
        Rule::return_statement => {
            let components = inner.into_inner();
            let expr = parse_expression(try_take_first(components)?)?;

            Statement::Return {
                expression: Box::new(expr),
            }
        }
        _ => {
            bail!("unsupported rule: {inner:?}")
        }
    };

    Ok(AstNode {
        kind: AstNodeKind::Statement(inner),
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ast::Literal, grammar::GrammarParser};
    use pest::Parser;

    #[test]
    fn test_parse_statement_expression() {
        let parsed = GrammarParser::parse(Rule::statement, "123;")
            .unwrap()
            .nth(0)
            .unwrap();
        let stmt = parse_statement(parsed).unwrap();
        let stmt = stmt.as_statement().unwrap();
        let exp = stmt.as_expression().unwrap();
        let exp = exp.as_expression().unwrap();
        let literal = exp.as_literal().unwrap().as_literal().unwrap();

        assert!(matches!(literal.0, Literal::Number));
    }

    #[test]
    fn test_parse_statement_declare_and_bind() {
        let parsed = GrammarParser::parse(Rule::statement, "let x: i32 = 123;")
            .unwrap()
            .nth(0)
            .unwrap();
        let stmt = parse_statement(parsed).unwrap();
        assert!(stmt.as_statement().unwrap().as_declare_and_bind().is_some());
    }

    #[test]
    fn test_parse_statement_bind() {
        let parsed = GrammarParser::parse(Rule::statement, "x = 124;")
            .unwrap()
            .nth(0)
            .unwrap();
        let stmt = parse_statement(parsed).unwrap();
        assert!(stmt.as_statement().unwrap().as_bind_ref().is_some());
    }

    #[test]
    fn test_parse_statement_return() {
        let parsed = GrammarParser::parse(Rule::statement, "return x;")
            .unwrap()
            .nth(0)
            .unwrap();
        let stmt = parse_statement(parsed).unwrap();
        assert!(stmt.as_statement().unwrap().as_return().is_some());
    }
}
