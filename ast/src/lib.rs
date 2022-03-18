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

pub fn get_env(
    env: Rc<RefCell<EnvInner>>,
    id: &Identifier,
    up: usize,
) -> Result<Rc<RefCell<Object>>> {
    let matching_env = EnvInner::_get_env(env, id, up)?;
    let matching_env = matching_env.borrow();
    match matching_env.values.get(&id.ident).unwrap() {
        Some(o) => Ok(Rc::clone(o)),
        None => Err(EnvErrorKind::UnintializedVariableAccessed(id.clone())),
    }
}

pub fn assign_env(
    env: Rc<RefCell<EnvInner>>,
    id: &Identifier,
    up: usize,
    value: Object,
) -> Result<()> {
    let matching_env = EnvInner::_get_env(env, id, up)?;
    let mut matching_env_mut = matching_env.borrow_mut();
    let old_value = matching_env_mut.values.get_mut(&id.ident).unwrap();
    *old_value = Some(Rc::new(RefCell::new(value)));
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

    #[error("Error at '{0}': Already a variable with this name in this scope.")]
    VariableExists(Identifier),

    #[error("Error at '{0}': Accessed an unintialized variable '{0}'.")]
    UnintializedVariableAccessed(Identifier),
}

type Result<T> = std::result::Result<T, EnvErrorKind>;
