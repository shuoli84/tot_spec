use crate::ast::ast::{AstNode, AstNodeKind, Literal};
use crate::ast::grammar::Rule;
use pest::iterators::Pair;

pub fn parse_literal(pair: Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::literal));

    let span = pair.as_span().into();

    let inner = pair.into_inner().nth(0).unwrap();

    let literal_kind = match inner.as_rule() {
        Rule::string_literal => Literal::String,
        Rule::number_literal => Literal::Number,
        Rule::bool_literal => Literal::Boolean,
        _ => {
            unreachable!()
        }
    };

    AstNode {
        kind: AstNodeKind::Literal {
            kind: literal_kind,
            value: inner.as_str().to_string(),
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
    fn test_parse_literal_bool() {
        let parsed = GrammarParser::parse(Rule::literal, "true")
            .unwrap()
            .nth(0)
            .unwrap();
        let literal = parse_literal(parsed);
        assert!(matches!(
            literal.kind,
            AstNodeKind::Literal {
                kind: Literal::Boolean,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_literal_number() {
        let parsed = GrammarParser::parse(Rule::literal, "123")
            .unwrap()
            .nth(0)
            .unwrap();
        let literal = parse_literal(parsed);
        assert!(matches!(
            literal.kind,
            AstNodeKind::Literal {
                kind: Literal::Number,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_literal_string() {
        let parsed = GrammarParser::parse(Rule::literal, "\"123\"")
            .unwrap()
            .nth(0)
            .unwrap();
        let literal = parse_literal(parsed);
        assert!(matches!(
            literal.kind,
            AstNodeKind::Literal {
                kind: Literal::String,
                ..
            }
        ));
    }
}
