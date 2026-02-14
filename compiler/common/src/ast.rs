use crate::span::Span;
use crate::types::Type;
use serde::{Deserialize, Serialize};

/// A complete G-Basic program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// A statement in the program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Let {
        name: Identifier,
        type_ann: Option<Type>,
        value: Expression,
        span: Span,
    },
    Function(FunctionDecl),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    For {
        variable: Identifier,
        iterable: Expression,
        body: Block,
        span: Span,
    },
    While {
        condition: Expression,
        body: Block,
        span: Span,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Return {
        value: Option<Expression>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Expression {
        expr: Expression,
        span: Span,
    },
    Block(Block),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Let { span, .. }
            | Statement::If { span, .. }
            | Statement::For { span, .. }
            | Statement::While { span, .. }
            | Statement::Match { span, .. }
            | Statement::Return { span, .. }
            | Statement::Break { span, .. }
            | Statement::Continue { span, .. }
            | Statement::Expression { span, .. } => *span,
            Statement::Function(f) => f.span,
            Statement::Block(b) => b.span,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: Identifier,
    pub type_ann: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    Literal(Literal),
    Identifier(Identifier),
    Wildcard(Span),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

/// An expression in G-Basic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(Identifier),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
        span: Span,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expression>,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
        span: Span,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    MethodChain {
        base: NamespaceRef,
        chain: Vec<MethodCall>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expression>,
        field: Identifier,
        span: Span,
    },
    Array {
        elements: Vec<Expression>,
        span: Span,
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
        span: Span,
    },
    /// String interpolation: `"Hello, {name}!"`
    StringInterp {
        parts: Vec<StringPart>,
        span: Span,
    },
    /// Range expression: `start..end`
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        span: Span,
    },
}

/// A part of an interpolated string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringPart {
    /// Literal text segment
    Lit(String),
    /// An interpolated expression: `{expr}`
    Expr(Expression),
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal(lit) => lit.span,
            Expression::Identifier(id) => id.span,
            Expression::BinaryOp { span, .. }
            | Expression::UnaryOp { span, .. }
            | Expression::Call { span, .. }
            | Expression::Index { span, .. }
            | Expression::MethodChain { span, .. }
            | Expression::FieldAccess { span, .. }
            | Expression::Array { span, .. }
            | Expression::Assignment { span, .. }
            | Expression::StringInterp { span, .. }
            | Expression::Range { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCall {
    pub method: Identifier,
    pub args: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamespaceRef {
    Screen,
    Sound,
    Input,
    Math,
    System,
    Memory,
    IO,
    Asset,
}

impl std::fmt::Display for NamespaceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceRef::Screen => write!(f, "Screen"),
            NamespaceRef::Sound => write!(f, "Sound"),
            NamespaceRef::Input => write!(f, "Input"),
            NamespaceRef::Math => write!(f, "Math"),
            NamespaceRef::System => write!(f, "System"),
            NamespaceRef::Memory => write!(f, "Memory"),
            NamespaceRef::IO => write!(f, "IO"),
            NamespaceRef::Asset => write!(f, "Asset"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiteralKind {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Neq => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}
