use std::collections::HashMap;

extern crate thiserror;
use thiserror::Error;

extern crate peekmore;
use peekmore::PeekMore;
use peekmore::PeekMoreIterator;

extern crate lazy_static;
use lazy_static::lazy_static;

mod lexer;
pub use lexer::Lexer;

mod token;
pub use token::Token;
pub use token::TokenType;

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

#[derive(Debug, Error)]
pub enum LexerErrorKind {
    #[error("Error: Unterminated string.")]
    UnterminatedStringLiteral,

    #[error("Error: Unexpected char '{ch}' found in input.")]
    UnexpectedChar { ch: char },
}

type Result<T> = std::result::Result<T, LexerErrorKind>;
