use crate::ast::grammar::{GrammarParser, Rule};
use pest::Parser;

mod block;
mod call;
mod expression;
mod expression_for;
mod expression_if;
mod expression_while;
mod func_def;
mod func_signature;
mod ident;
mod literal;
mod path;
mod statement;

#[derive(Debug)]
pub enum AstNodeKind {
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
        /// the last expression is the block's value
        value_expr: Option<Box<AstNode>>,
    },
    Expression(Expression),
    Statement(Statement),
    Literal {
        kind: Literal,
        value: String,
    },
    Reference {
        /// identifiers separated by .
        /// a.b.c => ["a", "b", "c"]
        identifiers: Vec<AstNode>,
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
    If {
        condition: Box<AstNode>,
        block: Box<AstNode>,
        else_block: Option<Box<AstNode>>,
    },
    /// for item in values {}
    For {
        item: Box<AstNode>,
        values: Box<AstNode>,
        block: Box<AstNode>,
    },
    /// while condition {}
    While {
        condition: Box<AstNode>,
        block: Box<AstNode>,
    },
    /// function call
    Call {
        path: Box<AstNode>,
        params: Vec<AstNode>,
    },
}

#[derive(Debug)]
pub struct AstNode {
    kind: AstNodeKind,
    span: Span,
}

impl AstNode {
    pub fn parse_statement(code: &str) -> anyhow::Result<Self> {
        let parsed = GrammarParser::parse(Rule::statement, code)
            .unwrap()
            .nth(0)
            .unwrap();
        let stmt = statement::parse_statement(parsed);
        Ok(stmt)
    }
}

impl AstNode {
    pub fn kind(&self) -> &AstNodeKind {
        &self.kind
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn is_func_def(&self) -> bool {
        matches!(self.kind, AstNodeKind::FuncDef { .. })
    }

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

    pub fn as_statement(&self) -> Option<&Statement> {
        match &self.kind {
            AstNodeKind::Statement(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_expression(&self) -> Option<&Expression> {
        match &self.kind {
            AstNodeKind::Expression(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_literal(&self) -> Option<(Literal, &str)> {
        match &self.kind {
            AstNodeKind::Literal { kind, value } => Some((*kind, value.as_str())),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<&[AstNode]> {
        match &self.kind {
            AstNodeKind::Reference { identifiers } => Some(identifiers),
            _ => None,
        }
    }

    pub fn as_block(&self) -> Option<(&[AstNode], Option<&AstNode>)> {
        match &self.kind {
            AstNodeKind::Block {
                statements,
                value_expr,
            } => Some((statements, value_expr.as_ref().map(|x| x.as_ref()))),
            _ => None,
        }
    }

    pub fn is_if(&self) -> bool {
        matches!(self.kind, AstNodeKind::If { .. })
    }

    pub fn as_if(&self) -> Option<(&AstNode, &AstNode, Option<&AstNode>)> {
        match &self.kind {
            AstNodeKind::If {
                condition,
                block,
                else_block,
            } => Some((condition, block, else_block.as_ref().map(|x| x.as_ref()))),
            _ => None,
        }
    }

    pub fn is_for(&self) -> bool {
        matches!(self.kind, AstNodeKind::For { .. })
    }

    pub fn is_while(&self) -> bool {
        matches!(self.kind, AstNodeKind::While { .. })
    }

    pub fn as_for(&self) -> Option<(&AstNode, &AstNode, &AstNode)> {
        match &self.kind {
            AstNodeKind::For {
                item,
                values,
                block,
            } => Some((item, values, block)),
            _ => None,
        }
    }

    pub fn as_while(&self) -> Option<(&AstNode, &AstNode)> {
        match &self.kind {
            AstNodeKind::While { condition, block } => Some((condition, block)),
            _ => None,
        }
    }

    pub fn is_call(&self) -> bool {
        matches!(self.kind, AstNodeKind::Call { .. })
    }

    pub fn as_call(&self) -> Option<(&AstNode, &[AstNode])> {
        match &self.kind {
            AstNodeKind::Call { path, params } => Some((path, params)),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub enum Literal {
    String,
    Number,
    Boolean,
}

#[derive(Debug)]
pub enum Expression {
    Literal(Box<AstNode>),
    Reference(Box<AstNode>),
    Call(Box<AstNode>),
    If(Box<AstNode>),
    For(Box<AstNode>),
    Block(Box<AstNode>),
}

impl Expression {
    pub fn as_literal(&self) -> Option<&AstNode> {
        match self {
            Expression::Literal(node) => Some(node.as_ref()),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<&AstNode> {
        match self {
            Expression::Reference(node) => Some(node.as_ref()),
            _ => None,
        }
    }

    pub fn as_call(&self) -> Option<&AstNode> {
        match self {
            Expression::Call(node) => Some(node.as_ref()),
            _ => None,
        }
    }

    pub fn as_if(&self) -> Option<&AstNode> {
        match self {
            Expression::If(node) => Some(node.as_ref()),
            _ => None,
        }
    }

    pub fn as_for(&self) -> Option<&AstNode> {
        match self {
            Expression::For(node) => Some(node.as_ref()),
            _ => None,
        }
    }

    pub fn as_block(&self) -> Option<&AstNode> {
        match self {
            Expression::Block(node) => Some(node.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    DeclareAndBind {
        ident: Box<AstNode>,
        path: Box<AstNode>,
        expression: Box<AstNode>,
    },
    Bind {
        ident: Box<AstNode>,
        expression: Box<AstNode>,
    },
    Return {
        expression: Box<AstNode>,
    },
    Expression(Box<AstNode>),
}

impl Statement {
    pub fn as_expression(&self) -> Option<&AstNode> {
        match self {
            Statement::Expression(inner) => Some(inner.as_ref()),
            _ => None,
        }
    }

    pub fn as_declare_and_bind(&self) -> Option<DeclareAndBind> {
        match self {
            Statement::DeclareAndBind {
                ident,
                path,
                expression,
            } => Some(DeclareAndBind {
                ident: ident.as_ref(),
                path: path.as_ref(),
                expression: expression.as_ref(),
            }),
            _ => None,
        }
    }

    pub fn as_bind_ref(&self) -> Option<BindRef> {
        match self {
            Statement::Bind { ident, expression } => Some(BindRef {
                ident: ident.as_ref(),
                expression: expression.as_ref(),
            }),
            _ => None,
        }
    }

    pub fn as_return(&self) -> Option<ReturnRef> {
        match self {
            Statement::Return { expression } => Some(ReturnRef {
                expression: expression.as_ref(),
            }),
            _ => None,
        }
    }
}

pub struct DeclareAndBind<'a> {
    ident: &'a AstNode,
    path: &'a AstNode,
    expression: &'a AstNode,
}

pub struct BindRef<'a> {
    ident: &'a AstNode,
    expression: &'a AstNode,
}

pub struct ReturnRef<'a> {
    expression: &'a AstNode,
}
