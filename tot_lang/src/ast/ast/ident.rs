use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;

pub fn parse_ident(pair: pest::iterators::Pair<Rule>) -> AstNode {
    AstNode {
        kind: AstNodeKind::Ident {
            value: pair.as_str().to_string(),
        },
        span: pair.as_span().into(),
    }
}
