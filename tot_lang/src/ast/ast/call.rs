use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::path::parse_path;
use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;
use pest::iterators::Pair;

pub fn parse_call(pair: Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::call_exp));
    let span = pair.as_span().into();

    let mut inner = pair.into_inner();
    let path = parse_path(inner.next().unwrap());
    let params = inner.map(|i| parse_expression(i)).collect();

    AstNode {
        kind: AstNodeKind::Call {
            path: Box::new(path),
            params,
        },
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_parse_call_with_params() {
        let parsed = GrammarParser::parse(Rule::call_exp, "a::b::c(i, 123)")
            .unwrap()
            .nth(0)
            .unwrap();
        let ast = parse_call(parsed);
        assert!(ast.is_call());

        let (_path, params) = ast.as_call().unwrap();
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_parse_call_without_params() {
        let parsed = GrammarParser::parse(Rule::call_exp, "a::b::c()")
            .unwrap()
            .nth(0)
            .unwrap();
        let ast = parse_call(parsed);
        assert!(ast.is_call());
        let (_path, params) = ast.as_call().unwrap();
        assert!(params.is_empty());
    }
}