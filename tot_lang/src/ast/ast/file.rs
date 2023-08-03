use super::*;
use pest::iterators::Pair;

pub fn parse_file(pair: Pair<Rule>) -> AstNode {
    let span = pair.as_span().into();
    let mut inner = pair.into_inner();

    let mut func_defs = vec![];
    for inner in inner {
        func_defs.push(func_def::parse_func_def(inner));
    }

    AstNode {
        kind: AstNodeKind::File {
            func_defs: func_defs,
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
    fn test_parse_file() {
        let parsed = GrammarParser::parse(
            Rule::file,
            r#"fn hello(arg_1: i32, arg_2: i32) -> i32 {
                // just returns 3
                3
            }"#,
        )
        .unwrap()
        .nth(0)
        .unwrap();

        let node = parse_file(parsed);
        assert!(node.is_file());
    }
}
