use crate::ast::ast::ident::parse_ident;
use crate::ast::ast::path::parse_path;
use crate::ast::ast::{try_next, try_take_first, AstNode, AstNodeKind, Span};
use crate::ast::grammar::Rule;
use anyhow::bail;

pub fn parse_func_signature(pair: pest::iterators::Pair<Rule>) -> anyhow::Result<AstNode> {
    assert!(matches!(pair.as_rule(), Rule::func_signature));

    let span: Span = pair.as_span().into();
    let mut inner = pair.into_inner();
    let ident = try_next(&mut inner)?;
    let ident_ast = parse_ident(ident);

    let mut params: Vec<AstNode> = vec![];
    let mut ret: Option<AstNode> = None;

    for inner in inner {
        match inner.as_rule() {
            Rule::func_param => {
                params.push(parse_func_param(inner)?);
            }
            Rule::func_ret => ret = Some(parse_func_ret(inner)?),
            _ => {
                bail!("unsupported rule: {inner:?}");
            }
        }
    }

    Ok(AstNode {
        kind: AstNodeKind::FuncSignature {
            name: Box::new(ident_ast),
            params,
            return_ty: ret.map(|i| Box::new(i)),
        },
        span,
    })
}

fn parse_func_param(pair: pest::iterators::Pair<Rule>) -> anyhow::Result<AstNode> {
    let span = pair.as_span().into();
    let mut inner = pair.into_inner();
    let param_ident = try_next(&mut inner)?;
    let ident = parse_ident(param_ident);
    let ty_path = parse_path(try_next(&mut inner)?);
    Ok(AstNode {
        kind: AstNodeKind::FuncParam {
            ident: Box::new(ident),
            ty: Box::new(ty_path),
        },
        span,
    })
}

fn parse_func_ret(pair: pest::iterators::Pair<Rule>) -> anyhow::Result<AstNode> {
    Ok(parse_path(try_take_first(pair.into_inner())?))
}

#[cfg(test)]
mod tests {
    use crate::ast::ast::func_signature::parse_func_signature;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_func_signature() {
        let successful_parse = GrammarParser::parse(
            Rule::func_signature,
            "fn hello(arg_1: i32, arg_2: i32) -> i32",
        )
        .unwrap()
        .nth(0)
        .unwrap();

        let ast = parse_func_signature(successful_parse).unwrap();
        let (name, params, ret) = ast.as_func_signature().unwrap();
        assert_eq!(name.as_ident().unwrap(), "hello");
        assert_eq!(params.len(), 2);
        assert_eq!(ret.unwrap().as_path().unwrap(), "i32");
    }

    #[test]
    fn test_func_signature_no_arg_no_ret() {
        let successful_parse = GrammarParser::parse(Rule::func_signature, "fn hello()")
            .unwrap()
            .nth(0)
            .unwrap();

        let ast = parse_func_signature(successful_parse).unwrap();
        let (name, params, ret) = ast.as_func_signature().unwrap();
        assert_eq!(name.as_ident().unwrap(), "hello");
        assert_eq!(params.len(), 0);
        assert!(ret.is_none());
    }
}
