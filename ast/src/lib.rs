use std::cell::RefCell;
use std::rc::Rc;

extern crate lox_lexer;
use lox_lexer::Span;
use lox_lexer::TokenType;

extern crate thiserror;
use thiserror::Error;

mod ast;
pub use ast::*;

mod env;
use env::EnvInner;
pub type Env = Rc<RefCell<EnvInner>>;

#[inline(always)]
pub fn new_env() -> Env {
    Rc::new(RefCell::new(EnvInner::new()))
}

#[inline(always)]
pub fn push_env(env: Env) -> Env {
    Rc::new(RefCell::new(EnvInner::detach_env(env)))
}

#[inline(always)]
pub fn pop_env(env: Env) -> Env {
    env.borrow()
        .enclosing
        .as_ref()
        .expect("no enclosing env")
        .clone()
}

#[inline(always)]
pub fn get_env(env: &EnvInner, id: &Identifier, up: usize) -> Result<Rc<RefCell<Object>>> {
    EnvInner::_get(env, id, up)
}

#[inline(always)]
pub fn assign_env(env: &EnvInner, id: &Identifier, up: usize, value: Object) -> Result<()> {
    let old_value = EnvInner::_get(env, id, up)?;
    *old_value.borrow_mut() = value;
    Ok(())
}

#[derive(Debug, Error)]
pub enum EnvErrorKind {
    #[error("Error: Undefined variable '{0}'.")]
    UndefinedVariable(Identifier),

    #[error("Error: Enclosing environment does not exist.")]
    NoEnclosingEnv,

    #[error("Undefined property '{0}'.")]
    UndefinedProperty(String),
}

type Result<T> = std::result::Result<T, EnvErrorKind>;
