use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
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
    Str(String),
    Numeric(String),
    Ident(String),
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
}

pub type Token = TokenType;

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenType::*;
        match self {
            LeftParen => write!(f, "("),
            RightParen => write!(f, ")"),
            LeftBrace => write!(f, "{{"),
            RightBrace => write!(f, "}}"),
            Dot => write!(f, "."),
            Comma => write!(f, ","),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            SemiColon => write!(f, ";"),
            ForwardSlash => write!(f, "/"),
            Star => write!(f, "*"),
            Not => write!(f, "!"),
            Ne => write!(f, "!="),
            Eq => write!(f, "="),
            Deq => write!(f, "=="),
            Gt => write!(f, ">"),
            Ge => write!(f, ">="),
            Lt => write!(f, "<"),
            Le => write!(f, "<="),
            Str(s) => write!(f, "\"{}\"", s),
            Numeric(num) => write!(f, "{}", num),
            Ident(id) => write!(f, "{}", id),
            True => write!(f, "true"),
            False => write!(f, "false"),
            And => write!(f, "&&"),
            Class => write!(f, "class"),
            Else => write!(f, "else"),
            For => write!(f, "for"),
            Fun => write!(f, "fn"),
            If => write!(f, "if"),
            Nil => write!(f, "nil"),
            Or => write!(f, "or"),
            Print => write!(f, "print"),
            Return => write!(f, "return"),
            Super => write!(f, "super"),
            This => write!(f, "this"),
            Var => write!(f, "var"),
            While => write!(f, "while"),
        }
    }
}
