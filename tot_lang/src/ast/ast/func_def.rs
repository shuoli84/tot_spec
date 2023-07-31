use super::*;

pub fn parse_func_def(
    pair: pest::iterators::Pair<Rule>,
    signature: AstNode,
    body: AstNode,
) -> AstNode {
    AstNode {
        kind: AstNodeKind::FuncDef {
            signature: Box::new(signature),
            body: Box::new(body),
        },
        span: pair.as_span().into(),
    }
}
