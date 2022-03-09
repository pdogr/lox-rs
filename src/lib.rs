use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::anyhow;

use thiserror::Error;

mod ast;
use ast::*;

mod callable;

mod env;
use env::EnvInner;
type Env = Rc<RefCell<EnvInner>>;

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

mod evaluator;
use evaluator::EvalResult;
use evaluator::Evaluator;

mod interpreter;
pub use interpreter::Interpreter;

mod lexer;

mod parser;

mod token;
use token::Token;
use token::TokenType;

extern crate peekmore;
use peekmore::PeekMore;
use peekmore::PeekMoreIterator;

extern crate lazy_static;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        vec![
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ]
        .into_iter()
        .collect()
    };
}

#[derive(Debug, Error)]
pub enum ErrorOrCtxJmp {
    #[error("{0}")]
    Error(anyhow::Error),

    #[error("encountered a RetJump, this is a BUG.")]
    RetJump { object: Object },
}

type Result<T> = std::result::Result<T, ErrorOrCtxJmp>;

#[cfg(test)]
mod test_utils {
    use std::cell::RefCell;
    use std::io::Write;
    use std::rc::Rc;

    #[derive(Debug, Clone)]
    pub(crate) struct TestWriter {
        inner: Rc<RefCell<Vec<u8>>>,
    }

    impl TestWriter {
        pub(crate) fn new() -> Self {
            TestWriter {
                inner: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn into_inner(self) -> Vec<u8> {
            Rc::try_unwrap(self.inner)
                .expect("TestWriter: More than one Rc refers to the inner Vec")
                .into_inner()
        }

        pub(crate) fn into_string(self) -> String {
            String::from_utf8(self.into_inner()).unwrap()
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.inner.borrow_mut().flush()
        }
    }
}
