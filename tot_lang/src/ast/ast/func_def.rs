use super::*;
use crate::ast::ast::block::parse_block;
use crate::ast::ast::func_signature::parse_func_signature;
use pest::iterators::Pair;

pub fn parse_func_def(pair: Pair<Rule>) -> AstNode {
    let span = pair.as_span().into();
    let mut inner = pair.into_inner();

    let signature = parse_func_signature(inner.next().unwrap());
    let block = parse_block(inner.next().unwrap());

    AstNode {
        kind: AstNodeKind::FuncDef {
            signature: Box::new(signature),
            body: Box::new(block),
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
        let parsed = GrammarParser::parse(Rule::func, "fn hello(arg_1: i32, arg_2: i32) -> i32 {}")
            .unwrap()
            .nth(0)
            .unwrap();

        parse_func_def(parsed);
    }
}
