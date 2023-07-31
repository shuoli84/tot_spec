use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;

pub fn parse_path(pair: pest::iterators::Pair<Rule>) -> AstNode {
    AstNode {
        kind: AstNodeKind::Path {
            value: pair.as_str().to_string(),
        },
        span: pair.as_span().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grammar::GrammarParser;
    use pest::Parser;

    #[test]
    fn test_parse_path() {
        let parsed = GrammarParser::parse(Rule::path, "a::b::c")
            .unwrap()
            .nth(0)
            .unwrap();
        let output = parse_path(parsed);
        dbg!(output);
    }
}
