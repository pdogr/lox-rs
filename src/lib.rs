use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::stdout;
use std::io::Write;
use std::rc::Rc;

use anyhow::anyhow;

use thiserror::Error;

mod ast;
use ast::*;

mod callable;

mod env;
use env::EnvInner;
type Env = Rc<RefCell<EnvInner>>;

fn new_env() -> Env {
    Rc::new(RefCell::new(EnvInner::new()))
}

fn push_env(env: Env) -> Env {
    Rc::new(RefCell::new(EnvInner::detach_env(env)))
}

fn pop_env(env: Env) -> Env {
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
use interpreter::Interpreter;

mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

mod resolver;
use resolver::Resolver;

mod token;
use token::Token;
use token::TokenType;

extern crate peekmore;
use peekmore::PeekMore;
use peekmore::PeekMoreIterator;

use rustyline::error::ReadlineError;
use rustyline::Editor;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate uuid;
use uuid::Uuid;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
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

fn prompt() {
    let mut interpreter = Interpreter::new(stdout());
    let mut resolver = Resolver::new();
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match runline(line, &mut interpreter, &mut resolver) {
                    Err(e) => {
                        println!("Error in repl: {}", e);
                        continue;
                    }
                    _ => continue,
                }
            }
            Err(ReadlineError::Eof | ReadlineError::Interrupted) => break,
            Err(e) => {
                println!("Error in repl: {}", e);
                continue;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}

fn runline<W: Write>(
    line: String,
    interpreter: &mut Interpreter<W>,
    resolver: &mut Resolver,
) -> Result<()> {
    let lexer = Lexer::new(line.chars()).unwrap();
    let tokens: Result<Vec<Token>> = lexer.into_iter().collect();
    let tokens = tokens?;
    let stmts = Parser::new(tokens.into_iter()).program()?;
    resolver.resolve(&stmts, interpreter)?;
    interpreter.run_many(stmts)?;
    Ok(())
}

fn runfile_stdout(file: &str) {
    let mut interpreter = Interpreter::new(stdout());
    match runfile(file, &mut interpreter) {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn runfile<W: Write>(file: &str, interpreter: &mut Interpreter<W>) -> Result<()> {
    let program = read_to_string(file).map_err(|e| {
        ErrorOrCtxJmp::Error(anyhow!("unable to read file {} with error {}", file, e))
    })?;
    let lexer = Lexer::new(program.chars()).unwrap();
    let tokens: Result<Vec<Token>> = lexer.into_iter().collect();
    let tokens = tokens?;
    let stmts = Parser::new(tokens.into_iter()).program()?;
    let mut resolver = Resolver::new();
    resolver.resolve(&stmts, interpreter)?;
    interpreter.run_many(stmts)
}

pub struct Runner {}

impl Runner {
    pub fn run(file: Option<&String>) {
        match file {
            Some(s) => runfile_stdout(s as &str),
            None => prompt(),
        }
    }
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
