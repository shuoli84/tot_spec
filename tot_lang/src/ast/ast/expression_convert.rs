use crate::ast::ast::block::parse_block;
use crate::ast::ast::expression::parse_reference;
use crate::ast::ast::path::parse_path;
use crate::ast::ast::{try_next, try_take_first, AstNode, AstNodeKind};
use crate::ast::grammar::Rule;

pub fn parse_convert(pair: pest::iterators::Pair<Rule>) -> anyhow::Result<AstNode> {
    assert!(matches!(pair.as_rule(), Rule::convert_exp));
    let span = pair.as_span().into();

    let mut inner = pair.into_inner();
    let reference = try_next(&mut inner)?;
    let expr = parse_reference(reference)?;

    let path = try_next(&mut inner)?;
    let path = parse_path(path);

    Ok(AstNode {
        kind: AstNodeKind::Convert {
            expr: Box::new(expr),
            target_path: Box::new(path),
        },
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grammar::GrammarParser;
    use pest::Parser;

    #[test]
    fn test_parse_convert() {
        let parsed = GrammarParser::parse(Rule::convert_exp, "a as i32")
            .unwrap()
            .nth(0)
            .unwrap();
        let ast = parse_convert(parsed).unwrap();
        assert!(ast.is_convert());
    }
}
