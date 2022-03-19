use std::fmt::{Debug, Display};

use crate::span::Span;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum TokenType {
    // Single char tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Dot,
    Comma,
    Plus,
    Minus,
    SemiColon,
    ForwardSlash,
    Star,

    // Double char tokens
    Not,
    Ne,
    Eq,
    Deq,
    Gt,
    Ge,
    Lt,
    Le,

    // Literals
    Str,
    Numeric,
    Ident,
    True,
    False,

    // Keywords
    And,
    Class,
    Else,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    Break,
    Continue,

    // Eof
    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenType::*;
        write!(
            f,
            "{}",
            match self {
                LeftParen => "(",
                RightParen => ")",
                LeftBrace => "{",
                RightBrace => "}",
                Dot => ".",
                Comma => ",",
                Plus => "+",
                Minus => "-",
                SemiColon => ";",
                ForwardSlash => "/",
                Star => "*",
                Not => "!",
                Ne => "!=",
                Eq => "=",
                Deq => "==",
                Gt => ">",
                Ge => ">=",
                Lt => "<",
                Le => "<=",
                True => "true",
                False => "false",
                And => "and",
                Class => "class",
                Else => "else",
                For => "for",
                Fun => "fn",
                If => "if",
                Nil => "nil",
                Or => "or",
                Print => "print",
                Return => "return",
                Super => "super",
                This => "this",
                Var => "var",
                While => "while",
                Break => "break",
                Continue => "continue",
                Eof => "<eof>",
                Str => "<str>",
                Numeric => "<numeric>",
                Ident => "<identifier>",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(ty: TokenType, span: Span) -> Self {
        use TokenType::*;
        let lexeme = match ty {
            LeftParen => "(",
            RightParen => ")",
            LeftBrace => "{",
            RightBrace => "}",
            Dot => ".",
            Comma => ",",
            Plus => "+",
            Minus => "-",
            SemiColon => ";",
            ForwardSlash => "/",
            Star => "*",
            Not => "!",
            Ne => "!=",
            Eq => "=",
            Deq => "==",
            Gt => ">",
            Ge => ">=",
            Lt => "<",
            Le => "<=",
            True => "true",
            False => "false",
            And => "and",
            Class => "class",
            Else => "else",
            For => "for",
            Fun => "fn",
            If => "if",
            Nil => "nil",
            Or => "or",
            Print => "print",
            Return => "return",
            Super => "super",
            This => "this",
            Var => "var",
            While => "while",
            Break => "break",
            Continue => "continue",
            Eof => "<eof>",
            _ => unreachable!(),
        };
        Self {
            ty,
            lexeme: lexeme.into(),
            span,
        }
    }

    pub fn new_with_lexeme(ty: TokenType, lexeme: &str, span: Span) -> Self {
        Self {
            ty,
            lexeme: lexeme.into(),
            span,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ty {
            TokenType::Numeric | TokenType::Ident | TokenType::Str => write!(f, "{}", self.lexeme),
            _ => write!(f, "{}", self.ty),
        }
    }
}
