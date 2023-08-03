use crate::ast::ast::block::parse_block;
use crate::ast::ast::expression::parse_expression;
use crate::ast::ast::ident::parse_ident;
use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;

pub fn parse_while(pair: pest::iterators::Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::while_exp));
    let span = pair.as_span().into();

    let mut inner = pair.into_inner();
    let condition = inner.next().unwrap();
    let condition = parse_expression(condition.into_inner().nth(0).unwrap());

    let while_block = inner.next().unwrap();
    let while_block = parse_block(while_block);

    AstNode {
        kind: AstNodeKind::While {
            condition: Box::new(condition),
            block: Box::new(while_block),
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
    fn test_parse_while() {
        let parsed = GrammarParser::parse(Rule::while_exp, "while true { print(1); }")
            .unwrap()
            .nth(0)
            .unwrap();
        let ast = parse_while(parsed);
        assert!(ast.is_while());
    }
}
