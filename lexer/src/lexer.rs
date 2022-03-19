use std::iter::Iterator;

use crate::token::*;
use crate::LexerErrorKind;
use crate::PeekMore;
use crate::PeekMoreIterator;
use crate::Result;
use crate::Span;
use crate::TokenType;
use crate::KEYWORDS;

pub struct Lexer<I: Iterator> {
    input: PeekMoreIterator<I>,
    span: Span,
}

impl<I: Iterator<Item = char>> Lexer<I> {
    pub fn new(input: I) -> Result<Self> {
        let lexer = Lexer {
            input: input.peekmore(),
            span: Span::new(1, 1),
        };

        Ok(lexer)
    }

    #[inline(always)]
    fn eof(&mut self) -> bool {
        self.input.peek().is_none()
    }

    #[inline(always)]
    fn match_nth<F>(&mut self, n: usize, f: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        if let Some(ch) = self.input.peek_nth(n) {
            if f(*ch) {
                return true;
            }
        }
        false
    }

    #[inline(always)]
    fn match_next(&mut self, c: char) -> bool {
        if let Some(ch) = self.input.peek() {
            if *ch == c {
                let t = self.input.next();
                if let Some('\n') = t {
                    self.span.newline();
                }

                return true;
            }
        }
        false
    }

    #[inline(always)]
    fn skip(&mut self, n: usize) {
        for _ in 0..n {
            let t = self.input.next();
            if let Some('\n') = t {
                self.span.newline();
            }
        }
    }

    #[inline(always)]
    fn skip_while<F>(&mut self, f: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(ch) = self.input.peek() {
            if f(*ch) {
                let t = self.input.next();
                if let Some('\n') = t {
                    self.span.newline();
                }
            } else {
                break;
            }
        }
    }

    #[inline(always)]
    fn take_while<F>(&mut self, f: F) -> Vec<char>
    where
        F: Fn(char) -> bool,
    {
        let mut taken = Vec::new();
        while let Some(ch) = self.input.peek() {
            if f(*ch) {
                taken.push(*ch);
                let t = self.input.next();
                if let Some('\n') = t {
                    self.span.newline();
                }
            } else {
                break;
            }
        }
        taken
    }

    #[inline(always)]
    fn make_token(&mut self, ty: TokenType) -> Result<Token> {
        let lexeme = format!("{}", ty);
        self.make_token_with_lexeme(ty, lexeme)
    }

    #[inline(always)]
    fn make_token_with_lexeme(&mut self, ty: TokenType, lexeme: String) -> Result<Token> {
        let len = lexeme.len();
        let token = Ok(Token {
            ty,
            lexeme,
            span: self.span,
        });
        self.span.advance_col(len);
        token
    }

    #[inline(always)]
    fn next_token(&mut self) -> Result<Token> {
        use TokenType::*;
        loop {
            match self.input.next() {
                Some(c) => match c {
                    '(' => return self.make_token(LeftParen),
                    ')' => return self.make_token(RightParen),
                    '{' => return self.make_token(LeftBrace),
                    '}' => return self.make_token(RightBrace),
                    '.' => return self.make_token(Dot),
                    ',' => return self.make_token(Comma),
                    '+' => return self.make_token(Plus),
                    '-' => return self.make_token(Minus),
                    ';' => return self.make_token(SemiColon),
                    '*' => return self.make_token(Star),
                    '/' => match self.match_next('/') {
                        true => {
                            self.skip_while(|c| c != '\n');
                            continue;
                        }
                        false => match self.match_next('*') {
                            true => loop {
                                self.skip_while(|c| c != '*');
                                self.skip(1);
                                if self.eof() {
                                    return Err(LexerErrorKind::UntermiatedBlockComment);
                                }
                                if self.match_next('/') {
                                    break;
                                }
                            },
                            false => return self.make_token(ForwardSlash),
                        },
                    },
                    '!' => {
                        return match self.match_next('=') {
                            true => self.make_token(Ne),
                            false => self.make_token(Not),
                        }
                    }
                    '=' => {
                        return match self.match_next('=') {
                            true => self.make_token(Deq),
                            false => self.make_token(Eq),
                        }
                    }
                    '<' => {
                        return match self.match_next('=') {
                            true => self.make_token(Le),
                            false => self.make_token(Lt),
                        }
                    }
                    '>' => {
                        return match self.match_next('=') {
                            true => self.make_token(Ge),
                            false => self.make_token(Gt),
                        }
                    }
                    ' ' | '\r' | '\t' => {
                        self.span.advance_col(1);
                        continue;
                    }
                    '\n' => {
                        self.span.newline();
                        continue;
                    }
                    '"' => {
                        let literal: String = self.take_while(|c| c != '"').into_iter().collect();
                        if !self.match_nth(0, |c| c == '"') {
                            return Err(LexerErrorKind::UnterminatedStringLiteral);
                        }
                        self.skip(1);
                        // For starting and ending double quotes as literl only contains unquoted
                        // string.
                        let token = self.make_token_with_lexeme(Str, literal);
                        self.span.advance_col(2);
                        return token;
                    }
                    d if d.is_ascii_digit() => {
                        let mut number = vec![c];
                        number.extend(self.take_while(|c| c.is_ascii_digit()));
                        if self.match_nth(0, |c| c == '.')
                            && self.match_nth(1, |c| c.is_ascii_digit())
                        {
                            number.push(self.input.next().expect("BUG"));
                            number.extend(self.take_while(|c| c.is_ascii_digit()));
                        }
                        return self.make_token_with_lexeme(Numeric, number.into_iter().collect());
                    }
                    a if a.is_ascii_alphanumeric() => {
                        let mut identifier = vec![a];
                        identifier
                            .extend(self.take_while(|c| c.is_ascii_alphanumeric() || c == '_'));
                        let identifier: String = identifier.into_iter().collect();
                        let ty = KEYWORDS.get(&identifier as &str).unwrap_or(&Ident);
                        return self.make_token_with_lexeme(*ty, identifier);
                    }
                    ch => {
                        return Err(LexerErrorKind::UnexpectedChar { ch });
                    }
                },
                None => return self.make_token(Eof),
            };
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(tok) if tok.ty == TokenType::Eof => None,
            x => Some(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    #[allow(unused_macros)]
    macro_rules! test_lexer_ok{
        ($name: ident,$input: literal,$($tk: expr),+ $(,)?) =>{
            #[test]
            fn $name(){
                use TokenType::*;
                let input=$input;
                let lexer = Lexer::new(input.chars()).unwrap();
                let tokens: Result<Vec<Token>> = lexer.into_iter().collect();

                dbg!($input);
                assert!(tokens.is_ok());
                assert_eq!(
                    tokens.unwrap(),
                    &[
                    $(
                        $tk
                    ),*
                    ]
                )
            }
        }
    }

    #[allow(unused_macros)]
    macro_rules! test_lexer_err {
        ($name: ident,$input: literal, $err: expr) => {
            #[test]
            fn $name() {
                let input = $input;
                let lexer = Lexer::new(input.chars()).unwrap();
                let tokens: Result<Vec<Token>> = lexer.into_iter().collect();

                assert!(tokens.is_err());
            }
        };
    }

    test_lexer_ok!(
        ignores_whitespace,
        "\r\r\t\t\t  ;   \t\t\t\n \n . + \r",
        Token::new(SemiColon, Span::new(1, 8)),
        Token::new(Dot, Span::new(3, 2)),
        Token::new(Plus, Span::new(3, 4))
    );

    test_lexer_ok!(
        single_char_tokens,
        ";> = < , {\n() } +-*/",
        Token::new(SemiColon, Span::new(1, 1)),
        Token::new(Gt, Span::new(1, 2)),
        Token::new(Eq, Span::new(1, 4)),
        Token::new(Lt, Span::new(1, 6)),
        Token::new(Comma, Span::new(1, 8)),
        Token::new(LeftBrace, Span::new(1, 10)),
        Token::new(LeftParen, Span::new(2, 1)),
        Token::new(RightParen, Span::new(2, 2)),
        Token::new(RightBrace, Span::new(2, 4)),
        Token::new(Plus, Span::new(2, 6)),
        Token::new(Minus, Span::new(2, 7)),
        Token::new(Star, Span::new(2, 8)),
        Token::new(ForwardSlash, Span::new(2, 9))
    );

    test_lexer_ok!(
        double_char_tokens,
        "== >= <= !=",
        Token::new(Deq, Span::new(1, 1)),
        Token::new(Ge, Span::new(1, 4)),
        Token::new(Le, Span::new(1, 7)),
        Token::new(Ne, Span::new(1, 10))
    );

    test_lexer_ok!(
        single_double_char_tokens,
        "==;.((}{))+/.",
        Token::new(Deq, Span::new(1, 1)),
        Token::new(SemiColon, Span::new(1, 3)),
        Token::new(Dot, Span::new(1, 4)),
        Token::new(LeftParen, Span::new(1, 5)),
        Token::new(LeftParen, Span::new(1, 6)),
        Token::new(RightBrace, Span::new(1, 7)),
        Token::new(LeftBrace, Span::new(1, 8)),
        Token::new(RightParen, Span::new(1, 9)),
        Token::new(RightParen, Span::new(1, 10)),
        Token::new(Plus, Span::new(1, 11)),
        Token::new(ForwardSlash, Span::new(1, 12)),
        Token::new(Dot, Span::new(1, 13)),
    );

    test_lexer_ok!(
        ignore_single_line_comment,
        "//Comment to be ignored.\n {}",
        Token::new(LeftBrace, Span::new(2, 2)),
        Token::new(RightBrace, Span::new(2, 3))
    );

    test_lexer_ok!(
        ignore_block_comment,
        r#"
        /* A block comment to be ignored
         * spanning many lines
         * and followed by a semicolon
         *
         * */
;
        "#,
        Token::new(SemiColon, Span::new(7, 1))
    );

    test_lexer_ok!(
        literal_str,
        "\"This is a string followed by a semi-colon.\";",
        Token::new_with_lexeme(
            Str,
            "This is a string followed by a semi-colon.",
            Span::new(1, 1)
        ),
        Token::new(SemiColon, Span::new(1, 45))
    );

    test_lexer_ok!(
        literal_int,
        "12 + 345; ",
        Token::new_with_lexeme(Numeric, "12", Span::new(1, 1)),
        Token::new(Plus, Span::new(1, 4)),
        Token::new_with_lexeme(Numeric, "345", Span::new(1, 6)),
        Token::new(SemiColon, Span::new(1, 9))
    );

    test_lexer_ok!(
        literal_float,
        "12.123123 + 345 ",
        Token::new_with_lexeme(Numeric, "12.123123", Span::new(1, 1)),
        Token::new(Plus, Span::new(1, 11)),
        Token::new_with_lexeme(Numeric, "345", Span::new(1, 13)),
    );

    test_lexer_ok!(
        lex_assignment,
        "a = 52;",
        Token::new_with_lexeme(Ident, "a", Span::new(1, 1)),
        Token::new(Eq, Span::new(1, 3)),
        Token::new_with_lexeme(Numeric, "52", Span::new(1, 5)),
        Token::new(SemiColon, Span::new(1, 7))
    );

    test_lexer_ok!(
        lex_keywords,
        "if (a=10) { return 1; }",
        Token::new(If, Span::new(1, 1)),
        Token::new(LeftParen, Span::new(1, 4)),
        Token::new_with_lexeme(Ident, "a", Span::new(1, 5)),
        Token::new(Eq, Span::new(1, 6)),
        Token::new_with_lexeme(Numeric, "10", Span::new(1, 7)),
        Token::new(RightParen, Span::new(1, 9)),
        Token::new(LeftBrace, Span::new(1, 11)),
        Token::new(Return, Span::new(1, 13)),
        Token::new_with_lexeme(Numeric, "1", Span::new(1, 20)),
        Token::new(SemiColon, Span::new(1, 21)),
        Token::new(RightBrace, Span::new(1, 23))
    );

    test_lexer_ok!(
        variable_decl,
        "var a;",
        Token::new_with_lexeme(Var, "var", Span::new(1, 1)),
        Token::new_with_lexeme(Ident, "a", Span::new(1, 5)),
        Token::new(SemiColon, Span::new(1, 6))
    );

    test_lexer_ok!(
        logical_op,
        "(1 or 2 and 3)",
        Token::new(LeftParen, Span::new(1, 1)),
        Token::new_with_lexeme(Numeric, "1", Span::new(1, 2)),
        Token::new(Or, Span::new(1, 4)),
        Token::new_with_lexeme(Numeric, "2", Span::new(1, 7)),
        Token::new(And, Span::new(1, 9)),
        Token::new_with_lexeme(Numeric, "3", Span::new(1, 13)),
        Token::new(RightParen, Span::new(1, 14))
    );

    test_lexer_ok!(
        break_stmt,
        "break;",
        Token::new(Break, Span::new(1, 1)),
        Token::new(SemiColon, Span::new(1, 6))
    );

    test_lexer_err!(
        unterminated_string_literal,
        "\" this string is not terminated",
        JLoxError::UnterminatedStringLiteral
    );

    test_lexer_err!(
        unterminated_block_comment,
        r#"
        var a=10;
        /* this block comment is unterminted.
         *
         *
         *
         *
        "#,
        JLoxError::UntermiatedBlockComment
    );
}
