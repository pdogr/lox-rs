use std::fmt::{Debug, Display};

use crate::{Env, Stmt};

#[derive(Clone)]
pub struct FuncInner {
    pub(crate) name: Option<String>,
    pub(crate) params: Vec<String>,
    pub(crate) body: Vec<Stmt>,
    pub(crate) closure: Env,
}

impl PartialEq for FuncInner {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.params == other.params && self.body == other.body
    }
}

impl Debug for FuncInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuncInner")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("body", &self.body)
            .finish()
    }
}

impl Display for FuncInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<fun: {}>({})",
            self.name.as_ref().unwrap_or(&"closure@".into()),
            self.params.join(",")
        )
    }
}

impl FuncInner {
    pub fn new(name: String, params: Vec<String>, body: Vec<Stmt>, closure: Env) -> Self {
        Self {
            name: Some(name),
            params,
            body,
            closure,
        }
    }

    pub fn new_lambda(params: Vec<String>, body: Vec<Stmt>, closure: Env) -> Self {
        Self {
            name: None,
            params,
            body,
            closure,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Nil,
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Func(FuncInner),
}

impl Eq for Object {}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Int(i) => write!(f, "{}", *i),
            Object::Float(fl) => write!(f, "{}", fl),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, r#""{}""#, s),
            Object::Func(fi) => write!(f, "{}", fi),
        }
    }
}

impl Object {
    pub fn is_truth(&self) -> bool {
        use Object::*;
        match self {
            Nil | Boolean(false) | Int(0) => false,
            _ => true,
        }
    }
}
