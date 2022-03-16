use std::cell::RefCell;
use std::rc::Rc;

extern crate lox_lexer;
use lox_lexer::TokenType;

extern crate thiserror;
use thiserror::Error;

extern crate uuid;
use uuid::Uuid;

mod ast;
pub use ast::*;

mod env;
use env::EnvInner;
pub type Env = Rc<RefCell<EnvInner>>;

pub fn new_env() -> Env {
    Rc::new(RefCell::new(EnvInner::new()))
}

pub fn push_env(env: Env) -> Env {
    Rc::new(RefCell::new(EnvInner::detach_env(env)))
}

pub fn pop_env(env: Env) -> Env {
    env.borrow()
        .enclosing
        .as_ref()
        .expect("no enclosing env")
        .clone()
}

#[derive(Debug, Error)]
pub enum EnvErrorKind {
    #[error("Error: Undefined variable '{0}'.")]
    UndefinedVariable(Identifier),

    #[error("Error: Enclosing environment does not exist.")]
    NoEnclosingEnv,

    #[error("Undefined property '{0}'.")]
    UndefinedProperty(String),

    #[error("Error at '{0}': Already a variable with this name in this scope.")]
    VariableExists(Identifier),
}

type Result<T> = std::result::Result<T, EnvErrorKind>;
