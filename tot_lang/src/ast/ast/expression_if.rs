use crate::ast::ast::block::parse_block;
use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;

pub fn parse_if(pair: pest::iterators::Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::if_exp));
    let span = pair.as_span().into();

    let mut inner = pair.into_inner();
    let if_condition = inner.next().unwrap();
    let if_condition = parse_expression(if_condition.into_inner().nth(0).unwrap());

    let if_block = inner.next().unwrap();
    let if_block = parse_block(if_block.into_inner().nth(0).unwrap());
    let mut else_block: Option<AstNode> = None;

    if let Some(pair) = inner.next() {
        assert!(matches!(pair.as_rule(), Rule::if_else_block));
        else_block = Some(parse_block(pair.into_inner().nth(0).unwrap()));
    }

    AstNode {
        kind: AstNodeKind::If {
            condition: Box::new(if_condition),
            block: Box::new(if_block),
            else_block: else_block.map(|n| Box::new(n)),
        },
        span,
    }
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
        let ast = parse_if(parsed);
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
        let ast = parse_if(parsed);
        assert!(ast.is_if());
        let (_cond, _block, else_block) = ast.as_if().unwrap();
        assert!(else_block.is_none());
    }
}
