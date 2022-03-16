use std::fs::read_to_string;
use std::io::stdout;
use std::io::Write;

extern crate anyhow;
use anyhow::anyhow;

extern crate lox_ast as ast;

extern crate lox_lexer as lexer;
pub use lexer::Lexer;

extern crate lox_parser as parser;
pub use parser::Parser;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

extern crate thiserror;
use thiserror::Error;

mod callable;

mod evaluator;
use evaluator::EvalResult;
use evaluator::Evaluator;

mod interpreter;
pub use interpreter::Interpreter;

mod resolver;
pub use resolver::Resolver;

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
    let lexer = lexer::Lexer::new(line.chars()).unwrap();
    let tokens: std::result::Result<Vec<lexer::Token>, _> = lexer.into_iter().collect();
    let tokens: Vec<lexer::Token> = tokens?;
    let stmts = parser::Parser::new(tokens.into_iter()).program()?;
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
    let tokens: std::result::Result<Vec<lexer::Token>, _> = lexer.into_iter().collect();
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
    Error(#[from] anyhow::Error),

    #[error("{0}")]
    ParserError(#[from] parser::ParserErrorKind),

    #[error("{0}")]
    LexerError(#[from] lexer::LexerErrorKind),

    #[error("{0}")]
    EnvError(#[from] ast::EnvErrorKind),

    #[error("encountered a RetJump, this is a BUG.")]
    RetJump { object: ast::Object },
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
