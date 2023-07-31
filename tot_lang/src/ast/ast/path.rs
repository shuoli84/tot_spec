use crate::ast::ast::{AstNode, AstNodeKind};
use crate::ast::grammar::Rule;

pub fn parse_path(pair: pest::iterators::Pair<Rule>) -> AstNode {
    AstNode {
        kind: AstNodeKind::Path {
            value: pair.as_str().to_string(),
        },
        span: pair.as_span().into(),
    }
}
