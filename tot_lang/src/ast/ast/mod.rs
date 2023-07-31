use crate::ast::grammar::Rule;

mod func_def;
mod func_signature;

#[derive(Debug)]
pub enum AstNodeKind {
    Ident {
        value: String,
    },
    Path {
        value: String,
    },
    FuncDef {
        signature: Box<AstNode>,
        body: Box<AstNode>,
    },

    FuncSignature {
        name: Box<AstNode>,
        params: Vec<AstNode>,
        return_ty: Option<Box<AstNode>>,
    },

    FuncParam {
        ident: Box<AstNode>,
        ty: Box<AstNode>,
    },
}

#[derive(Debug)]
pub struct AstNode {
    kind: AstNodeKind,
    span: Span,
}

impl AstNode {
    pub fn as_func_signature(&self) -> Option<(&AstNode, &[AstNode], Option<&AstNode>)> {
        match &self.kind {
            AstNodeKind::FuncSignature {
                name,
                params,
                return_ty,
            } => Some((&name, &params, return_ty.as_ref().map(|t| t.as_ref()))),
            _ => None,
        }
    }

    pub fn as_ident(&self) -> Option<&str> {
        match &self.kind {
            AstNodeKind::Ident { value } => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn as_path(&self) -> Option<&str> {
        match &self.kind {
            AstNodeKind::Path { value } => Some(value.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Span {
    start: usize,
    end: usize,
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span) -> Self {
        Self {
            start: value.start(),
            end: value.end(),
        }
    }
}

#[derive(Debug)]
pub struct Block {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grammar::{GrammarParser, Rule};
    use pest::Parser;

    #[test]
    fn test_grammar() {
        let successful_parse = GrammarParser::parse(Rule::file, "fn hello() {}").unwrap();
        // for file in successful_parse {
        //     for func in file.into_inner() {
        //         match func {
        //             Rule::func => parse_func_def(func.into_inner()),
        //         }
        //         if !matches!(func.as_rule(), Rule::func) {
        //             continue;
        //         }
        //
        //         let mut inner_rules = func.into_inner();
        //         let func_signature = inner_rules.next();
        //         let func_body = inner_rules.next();
        //
        //         dbg!(func_signature);
        //         dbg!(func_body);
        //     }
        // }
    }
}
