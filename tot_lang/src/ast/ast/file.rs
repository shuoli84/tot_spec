use super::*;
use pest::iterators::Pair;

pub fn parse_file(pair: Pair<Rule>) -> anyhow::Result<AstNode> {
    let span = pair.as_span().into();
    let inner = pair.into_inner();

    let mut func_defs = vec![];
    for inner in inner {
        func_defs.push(func_def::parse_func_def(inner)?);
    }

    Ok(AstNode {
        kind: AstNodeKind::File { func_defs },
        span,
    })
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

        let node = parse_file(parsed).unwrap();
        assert!(node.is_file());
        assert_eq!(node.as_file().unwrap().len(), 1)
    }
}
