use super::*;
use crate::ast::ast::block::parse_block;
use crate::ast::ast::func_signature::parse_func_signature;
use pest::iterators::Pair;

pub fn parse_func_def(pair: Pair<Rule>) -> anyhow::Result<AstNode> {
    let span = pair.as_span().into();
    let mut inner = pair.into_inner();

    let signature = parse_func_signature(try_next(&mut inner)?)?;

    let func_body = try_next(&mut inner)?;
    assert!(matches!(func_body.as_rule(), Rule::func_body));

    let body_block = parse_block(try_take_first(func_body.into_inner())?)?;

    Ok(AstNode {
        kind: AstNodeKind::FuncDef {
            signature: Box::new(signature),
            body: Box::new(body_block),
        },
        span,
    })
}

#[cfg(test)]
mod tests {
    use crate::ast::ast::func_def::parse_func_def;
    use crate::ast::ast::try_take_first;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_parse_func_def() {
        let parsed = try_take_first(
            GrammarParser::parse(
                Rule::func,
                r#"fn hello(arg_1: i32, arg_2: i32) -> i32 {
                // just returns 3
                3
            }"#,
            )
            .unwrap(),
        )
        .unwrap();

        let node = parse_func_def(parsed).unwrap();
        assert!(node.is_func_def());
    }
}
