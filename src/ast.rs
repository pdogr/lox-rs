use std::fmt::Display;

use crate::{stmt::Stmt, TokenType};

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

impl From<&TokenType> for BinaryOp {
    fn from(ttype: &TokenType) -> Self {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Int(i64),
    Float(f64),
    Boolean(bool),
    Ident(String),
    String(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Logical(BinaryOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Lambda(Vec<String>, Vec<Stmt>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Nil => write!(f, "nil"),
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Float(fl) => write!(f, "{}", fl),
            Expr::Boolean(b) => write!(f, "{}", b),
            Expr::Ident(s) => write!(f, "{}", s),
            Expr::String(s) => write!(f, r#""{}""#, s),
            Expr::Unary(uop, e) => write!(f, "({} {})", uop, e),
            Expr::Binary(bop, e1, e2) => write!(f, "({} {} {})", bop, e1, e2),
            Expr::Assign(ident, e) => write!(f, "(= {} {})", ident, e),
            Expr::Logical(bop, e1, e2) => write!(f, "({} {} {})", bop, e1, e2),
            Expr::Call(_callee, _args) => todo!(),
            Expr::Lambda(_args, _l) => todo!(),
        }
    }
}
