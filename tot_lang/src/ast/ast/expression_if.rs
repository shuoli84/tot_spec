use crate::ast::ast::block::parse_block;
use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::{try_next, try_take_first, AstNode, AstNodeKind};
use crate::ast::grammar::Rule;


pub fn parse_if(pair: pest::iterators::Pair<Rule>) -> anyhow::Result<AstNode> {
    assert!(matches!(pair.as_rule(), Rule::if_exp));
    let span = pair.as_span().into();

    let mut inner = pair.into_inner();
    let if_condition = try_next(&mut inner)?;
    let if_condition = parse_expression(try_take_first(if_condition.into_inner())?)?;

    let if_block = try_next(&mut inner)?;
    let if_block = parse_block(try_take_first(if_block.into_inner())?)?;
    let mut else_block: Option<AstNode> = None;

    if let Some(pair) = inner.next() {
        assert!(matches!(pair.as_rule(), Rule::if_else_block));
        else_block = Some(parse_block(try_take_first(pair.into_inner())?)?);
    }

    Ok(AstNode {
        kind: AstNodeKind::If {
            condition: Box::new(if_condition),
            block: Box::new(if_block),
            else_block: else_block.map(|n| Box::new(n)),
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
    fn test_parse_if_else() {
        let parsed = GrammarParser::parse(Rule::if_exp, "if true {} else {}")
            .unwrap()
            .nth(0)
            .unwrap();
        let ast = parse_if(parsed).unwrap();
        assert!(ast.is_if());

        let (_cond, _block, else_block) = ast.as_if().unwrap();
        assert!(else_block.is_some());
    }

    #[test]
    fn test_parse_if_no_else() {
        let parsed = GrammarParser::parse(Rule::if_exp, "if true {}")
            .unwrap()
            .nth(0)
            .unwrap();
        let ast = parse_if(parsed).unwrap();
        assert!(ast.is_if());
        let (_cond, _block, else_block) = ast.as_if().unwrap();
        assert!(else_block.is_none());
    }
}
