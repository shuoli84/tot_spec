use crate::ast::ast::block::parse_block;
use crate::ast::ast::call::parse_call;
use crate::ast::ast::expression_for::parse_for;
use crate::ast::ast::expression_if::parse_if;
use crate::ast::ast::ident::parse_ident;
use crate::ast::ast::literal::parse_literal;
use crate::ast::ast::{AstNode, AstNodeKind, Expression};
use crate::ast::grammar::Rule;
use pest::iterators::Pair;

pub fn parse_expression(pair: Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::expression));

    let span = pair.as_span().into();

    let inner = pair.into_inner().nth(0).unwrap();
    let expression = match inner.as_rule() {
        Rule::literal => Expression::Literal(Box::new(parse_literal(inner))),
        Rule::block => Expression::Block(Box::new(parse_block(inner))),
        Rule::reference => {
            let span = inner.as_span().into();
            Expression::Reference(Box::new(AstNode {
                kind: AstNodeKind::Reference {
                    identifiers: inner.into_inner().map(|id| parse_ident(id)).collect(),
                },
                span,
            }))
        }
        Rule::call_exp => Expression::Call(Box::new(parse_call(inner))),
        Rule::if_exp => Expression::If(Box::new(parse_if(inner))),
        Rule::for_exp => Expression::For(Box::new(parse_for(inner))),
        _ => {
            unreachable!();
        }
    };

    AstNode {
        kind: AstNodeKind::Expression(expression),
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ast::Literal;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_parse_expression_literal() {
        let parsed = GrammarParser::parse(Rule::expression, "123")
            .unwrap()
            .nth(0)
            .unwrap();
        let exp = parse_expression(parsed);
        let literal = exp
            .as_expression()
            .unwrap()
            .as_literal()
            .unwrap()
            .as_literal()
            .unwrap();
        assert!(matches!(literal.0, Literal::Number));
    }

    #[test]
    fn test_parse_expression_reference() {
        let parsed = GrammarParser::parse(Rule::expression, "x.y")
            .unwrap()
            .nth(0)
            .unwrap();
        let exp = parse_expression(parsed);
        let reference = exp
            .as_expression()
            .unwrap()
            .as_reference()
            .unwrap()
            .as_reference()
            .unwrap();
        assert_eq!(reference.len(), 2);
    }

    #[test]
    fn test_parse_expression_call() {
        let parsed = GrammarParser::parse(Rule::expression, "a::b()")
            .unwrap()
            .nth(0)
            .unwrap();
        let exp = parse_expression(parsed);
        assert!(exp.as_expression().unwrap().as_call().is_some());
    }

    #[test]
    fn test_parse_expression_if() {
        let parsed = GrammarParser::parse(Rule::expression, "if true {}")
            .unwrap()
            .nth(0)
            .unwrap();
        let exp = parse_expression(parsed);
        assert!(exp.as_expression().unwrap().as_if().is_some());
    }

    #[test]
    fn test_parse_expression_for() {
        let parsed = GrammarParser::parse(Rule::expression, "for i in values {}")
            .unwrap()
            .nth(0)
            .unwrap();
        let exp = parse_expression(parsed);
        assert!(exp.as_expression().unwrap().as_for().is_some());
    }

    #[test]
    fn test_parse_expression_block() {
        let parsed = GrammarParser::parse(Rule::expression, "{}")
            .unwrap()
            .nth(0)
            .unwrap();
        dbg!(&parsed);
        let exp = parse_expression(parsed);
        assert!(exp.as_expression().unwrap().as_block().is_some());
    }
}
