extern crate lox_ast as ast;

extern crate lox_lexer as lexer;

extern crate thiserror;
use thiserror::Error;

mod parser;
pub use parser::Parser;

#[derive(Debug, Error)]
pub enum ParserErrorKind {
    #[error("{0}")]
    EnvError(#[from] ast::EnvErrorKind),

    #[error("Error: missing token.")]
    MissingToken,

    #[error("Error at '{0}': {1}")]
    UnexpectedToken(lexer::Token, String),

    #[error("{0}")]
    MissingTokenWithMsg(String),

    #[error("Error at '{0}': Expect '{{' before function body.")]
    FunctionMissingLBraceFound(lexer::Token),

    #[error("Expect '{{' before function body.")]
    FunctionMissingLBrace,

    #[error("Error at '{0}': {1}")]
    ExpectedIdentifierNotFound(lexer::Token, String),

    #[error("Error at '{0}': Can't have more than 255 parameters.")]
    ExcessParamtersFound(lexer::Token),

    #[error("Error at '{0}': Can't have more than 255 arguments.")]
    ExcessArgumentsFound(lexer::Token),

    #[error("Error at '{0}': Already a variable with this name in this scope.")]
    DuplicateParamter(String),

    #[error("Error at '{0}': Expect expression.")]
    ExpectExpressionFound(String),

    #[error("Error at '{0}': Unable to parse ast float due to {1}.")]
    ParseFloatError(String, std::num::ParseFloatError),
}

type Result<T> = std::result::Result<T, ParserErrorKind>;
