use super::*;
use crate::ast::ast::block::parse_block;
use crate::ast::ast::func_signature::parse_func_signature;
use pest::iterators::Pair;

pub fn parse_func_def(pair: Pair<Rule>) -> AstNode {
    let span = pair.as_span().into();
    let mut inner = pair.into_inner();

    let signature = parse_func_signature(inner.next().unwrap());

    let func_body = inner.next().unwrap();
    assert!(matches!(func_body.as_rule(), Rule::func_body));

    let body_block = parse_block(func_body.into_inner().nth(0).unwrap());

    AstNode {
        kind: AstNodeKind::FuncDef {
            signature: Box::new(signature),
            body: Box::new(body_block),
        },
        span,
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::ast::func_def::parse_func_def;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_parse_func_def() {
        let parsed = GrammarParser::parse(
            Rule::func,
            r#"fn hello(arg_1: i32, arg_2: i32) -> i32 {
                // just returns 3
                3
            }"#,
        )
        .unwrap()
        .nth(0)
        .unwrap();

        let node = parse_func_def(parsed);
        assert!(node.is_func_def());
    }
}
