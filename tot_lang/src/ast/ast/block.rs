use super::*;
use pest::iterators::Pair;

pub fn parse_block(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();

    // dbg!(pair);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_parse_block() {
        let parsed = GrammarParser::parse(Rule::block, "{}")
            .unwrap()
            .nth(0)
            .unwrap();
        let block = parse_block(parsed);
    }
}
