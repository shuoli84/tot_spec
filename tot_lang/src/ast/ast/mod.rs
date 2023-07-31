use crate::ast::grammar::Rule;

mod block;
mod expression;
mod func_def;
mod func_signature;
mod ident;
mod literal;
mod path;
mod statement;

#[derive(Debug)]
pub enum AstNodeKind {
    DeclareAndBind {
        name: Box<AstNode>,
        ty_: Box<AstNode>,
        expression: Box<AstNode>,
    },
    Bind {
        name: Box<AstNode>,
        expression: Box<AstNode>,
    },
    Ident {
        value: String,
    },
    Path {
        value: String,
    },
    Block {
        statements: Vec<AstNode>,
    },
    Expression(Expression),
    Statement {},
    Literal {
        kind: Literal,
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
pub enum Literal {
    String,
    Number,
    Boolean,
}

#[derive(Debug)]
pub enum Expression {
    Literal(Box<AstNode>),
    Block(Box<AstNode>),
}

#[derive(Debug)]
pub enum Statement {
    DeclareAndBind(Box<AstNode>),
    Bind(Box<AstNode>),
    Expression(Box<AstNode>),
}
