use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::ident::parse_ident;
use crate::ast::ast::path::parse_path;
use crate::ast::ast::{AstNode, AstNodeKind, Statement};
use crate::ast::grammar::Rule;
use pest::iterators::Pair;

pub fn parse_statement(pair: Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::statement));

    let span = pair.as_span().into();
    let inner = pair.into_inner().nth(0).unwrap();

    let inner = match inner.as_rule() {
        Rule::expression => Statement::Expression(Box::new(parse_expression(inner))),
        Rule::declare_and_bind => {
            let mut components = inner.into_inner();
            let ident = components.next().unwrap();
            let ident = parse_ident(ident);
            let path = components.next().unwrap();
            let path = parse_path(path);
            let expression = components.next().unwrap();
            let expr = parse_expression(expression);

            Statement::DeclareAndBind {
                ident: Box::new(ident),
                path: Box::new(path),
                expression: Box::new(expr),
            }
        }
        Rule::bind => {
            let mut components = inner.into_inner();
            let ident = components.next().unwrap();
            let ident = parse_ident(ident);
            let expression = components.next().unwrap();
            let expr = parse_expression(expression);

            Statement::Bind {
                ident: Box::new(ident),
                expression: Box::new(expr),
            }
        }
        Rule::return_statement => {
            let mut components = inner.into_inner();
            let expr = parse_expression(components.next().unwrap());

            Statement::Return {
                expression: Box::new(expr),
            }
        }
        _ => {
            unreachable!()
        }
    };

    AstNode {
        kind: AstNodeKind::Statement(inner),
        span,
    }
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
        let stmt = parse_statement(parsed);
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
        let stmt = parse_statement(parsed);
        assert!(stmt.as_statement().unwrap().as_declare_and_bind().is_some());
    }

    #[test]
    fn test_parse_statement_bind() {
        let parsed = GrammarParser::parse(Rule::statement, "x = 124;")
            .unwrap()
            .nth(0)
            .unwrap();
        let stmt = parse_statement(parsed);
        assert!(stmt.as_statement().unwrap().as_bind_ref().is_some());
    }

    #[test]
    fn test_parse_statement_return() {
        let parsed = GrammarParser::parse(Rule::statement, "return x;")
            .unwrap()
            .nth(0)
            .unwrap();
        let stmt = parse_statement(parsed);
        assert!(stmt.as_statement().unwrap().as_return().is_some());
    }
}
