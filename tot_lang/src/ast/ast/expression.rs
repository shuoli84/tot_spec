use crate::ast::ast::literal::parse_literal;
use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;
use pest::iterators::Pair;

pub fn parse_expression(pair: Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::expression));

    let span = pair.as_span().into();

    let inner = pair.into_inner().nth(0).unwrap();
    let inner = match inner.as_rule() {
        Rule::literal => parse_literal(inner),
        Rule::block => {
            todo!()
        }
        _ => {
            unreachable!();
        }
    };

    AstNode {
        kind: AstNodeKind::Expression(Box::new(inner)),
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
        let literal = exp.as_expression().unwrap().as_literal().unwrap();
        assert!(matches!(literal.0, Literal::Number));
    }
}
