use super::*;
use crate::ast::ast::ident::parse_ident;

pub fn parse_reference(pair: pest::iterators::Pair<Rule>) -> AstNode {
    assert!(matches!(pair.as_rule(), Rule::reference));

    let span = pair.as_span().into();
    let inner = pair.into_inner();
    let identifiers = inner.map(|inner| parse_ident(inner)).collect();

    AstNode {
        kind: AstNodeKind::Reference { identifiers },
        span,
    }
}

// pub fn parse_reference(inner: Pair<Rule>) -> anyhow::Result<AstNode> {
//     let span = inner.as_span().into();
//     Ok(AstNode {
//         kind: AstNodeKind::Reference {
//             identifiers: inner.into_inner().map(|id| parse_ident(id)).collect(),
//         },
//         span,
//     })
// }
