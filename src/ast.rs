use std::fmt::Debug;
use std::fmt::Display;

use crate::Env;
use crate::TokenType;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum UnaryOp {
    Minus,
    Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            UnaryOp::Minus => "-",
            UnaryOp::Not => "!",
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum BinaryOp {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Lt,  // <
    Gt,  // >
    Eq,  // ==
    Le,  // <=
    Ge,  // >=
    Ne,  // !=
    Or,  // ||
    And, // &&
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Eq => "==",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::Ne => "!=",
            BinaryOp::Or => "or",
            BinaryOp::And => "and",
        };
        write!(f, "{}", c)
    }
}

impl From<TokenType> for BinaryOp {
    fn from(ttype: TokenType) -> Self {
        use TokenType::*;
        match ttype {
            Plus => BinaryOp::Add,
            Minus => BinaryOp::Sub,
            Star => BinaryOp::Mul,
            ForwardSlash => BinaryOp::Div,
            Lt => BinaryOp::Lt,
            Gt => BinaryOp::Gt,
            Le => BinaryOp::Le,
            Ge => BinaryOp::Ge,
            Deq => BinaryOp::Eq,
            Ne => BinaryOp::Ne,
            Or => BinaryOp::Or,
            And => BinaryOp::And,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Identifier(String);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub(crate) value: Expr,
}

impl From<Expr> for Argument {
    fn from(e: Expr) -> Self {
        Self { value: e }
    }
}

impl From<Argument> for Expr {
    fn from(a: Argument) -> Self {
        a.value
    }
}

pub type Arguments = Vec<Argument>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Int(i64),
    Float(f64),
    Boolean(bool),
    Ident(Identifier),
    String(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Logical(BinaryOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Arguments),
    Lambda(Vec<Identifier>, Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub(crate) name: Identifier,
    pub(crate) definition: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub(crate) name: Identifier,
    pub(crate) params: Vec<Identifier>,
    pub(crate) body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub(crate) cond: Expr,
    pub(crate) if_branch: Box<Stmt>,
    pub(crate) else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    pub(crate) cond: Expr,
    pub(crate) body: Box<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    VariableDecl(VariableDecl),
    Block(Vec<Stmt>),
    Conditional(Conditional),
    Loop(Loop),
    FunctionDecl(FunctionDecl),
    Return(Expr),
}

#[derive(Clone)]
pub struct FuncObject {
    pub(crate) name: Option<Identifier>,
    pub(crate) params: Vec<Identifier>,
    pub(crate) body: Vec<Stmt>,
    pub(crate) closure: Env,
}

impl FuncObject {
    pub fn new(name: Identifier, params: Vec<Identifier>, body: Vec<Stmt>, closure: Env) -> Self {
        Self {
            name: Some(name),
            params,
            body,
            closure,
        }
    }

    pub fn new_lambda(params: Vec<Identifier>, body: Vec<Stmt>, closure: Env) -> Self {
        Self {
            name: None,
            params,
            body,
            closure,
        }
    }
}

impl PartialEq for FuncObject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.params == other.params && self.body == other.body
    }
}

impl Debug for FuncObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuncInner")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("body", &self.body)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Nil,
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Function(FuncObject),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Int(i) => write!(f, "{}", *i),
            Object::Float(fl) => write!(f, "{}", *fl),
            Object::Boolean(b) => write!(f, "{}", *b),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Function(_) => todo!(),
        }
    }
}

impl Object {
    pub fn is_truth(&self) -> bool {
        use Object::*;
        !matches!(self, Nil | Boolean(false) | Int(0))
    }
}
