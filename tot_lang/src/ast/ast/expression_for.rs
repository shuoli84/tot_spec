use crate::ast::ast::block::parse_block;
use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::ident::parse_ident;
use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;

pub fn parse_for(pair: pest::iterators::Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::for_exp));
    let span = pair.as_span().into();

    dbg!(&pair);
    let mut inner = pair.into_inner();
    let item = inner.next().unwrap();
    let item = parse_ident(item.into_inner().nth(0).unwrap());

    let values = inner.next().unwrap();
    let values = parse_expression(values.into_inner().nth(0).unwrap());

    let for_block = inner.next().unwrap();
    let for_block = parse_block(for_block);

    AstNode {
        kind: AstNodeKind::For {
            item: Box::new(item),
            values: Box::new(values),
            block: Box::new(for_block),
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
    fn test_parse_for() {
        let parsed = GrammarParser::parse(Rule::for_exp, "for i in items {}")
            .unwrap()
            .nth(0)
            .unwrap();
        let ast = parse_for(parsed);
        assert!(ast.is_for());
    }
}
