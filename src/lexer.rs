use std::iter::Iterator;

use crate::anyhow;
use crate::token::*;
use crate::ErrorOrCtxJmp;
use crate::PeekMore;
use crate::PeekMoreIterator;
use crate::Result;
use crate::TokenType;
use crate::KEYWORDS;

pub struct Lexer<I: Iterator> {
    input: PeekMoreIterator<I>,
    lineno: usize,
}

impl<I: Iterator<Item = char>> Lexer<I> {
    pub fn new(input: I) -> Result<Self> {
        let lexer = Lexer {
            input: input.peekmore(),
            lineno: 0,
        };

        Ok(lexer)
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
                self.input.next();
                return true;
            }
        }
        false
    }

    #[inline(always)]
    fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.input.next();
        }
    }

    #[inline(always)]
    fn skip_while<F>(&mut self, f: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(ch) = self.input.peek() {
            if f(*ch) {
                self.input.next();
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
                self.input.next();
            } else {
                break;
            }
        }
        taken
    }

    fn next_token(&mut self) -> Result<Token> {
        use TokenType::*;
        loop {
            match self.input.next() {
                Some(c) => match c {
                    '(' => return Ok(Token::new(LeftParen)),
                    ')' => return Ok(Token::new(RightParen)),
                    '{' => return Ok(Token::new(LeftBrace)),
                    '}' => return Ok(Token::new(RightBrace)),
                    '.' => return Ok(Token::new(Dot)),
                    ',' => return Ok(Token::new(Comma)),
                    '+' => return Ok(Token::new(Plus)),
                    '-' => return Ok(Token::new(Minus)),
                    ';' => return Ok(Token::new(SemiColon)),
                    '*' => return Ok(Token::new(Star)),
                    '/' => match self.match_next('/') {
                        true => {
                            self.skip_while(|c| c != '\n');
                            continue;
                        }
                        false => return Ok(Token::new(ForwardSlash)),
                    },
                    '!' => {
                        return Ok(match self.match_next('=') {
                            true => Token::new(Ne),
                            false => Token::new(Not),
                        })
                    }
                    '=' => {
                        return Ok(match self.match_next('=') {
                            true => Token::new(Deq),
                            false => Token::new(Eq),
                        })
                    }
                    '<' => {
                        return Ok(match self.match_next('=') {
                            true => Token::new(Le),
                            false => Token::new(Lt),
                        })
                    }
                    '>' => {
                        return Ok(match self.match_next('=') {
                            true => Token::new(Ge),
                            false => Token::new(Gt),
                        })
                    }
                    ' ' | '\r' | '\t' => {
                        continue;
                    }
                    '\n' => {
                        self.lineno += 1;
                        continue;
                    }
                    '"' => {
                        let literal: String = self.take_while(|c| c != '"').into_iter().collect();
                        if !self.match_nth(0, |c| c == '"') {
                            return Err(ErrorOrCtxJmp::Error(anyhow!(
                                "Error: Unterminated string."
                            )));
                        }
                        self.skip(1);
                        return Ok(Token::new_with_lexeme(Str, &literal));
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
                        let numeric: String = number.into_iter().collect();
                        return Ok(Token::new_with_lexeme(Numeric, &numeric));
                    }
                    a if a.is_ascii_alphanumeric() => {
                        let mut identifier = vec![a];
                        identifier
                            .extend(self.take_while(|c| c.is_ascii_alphanumeric() || c == '_'));
                        let identifier: String = identifier.into_iter().collect();
                        let ty = KEYWORDS.get(&identifier as &str).unwrap_or(&Ident);
                        return Ok(Token::new_with_lexeme(*ty, &identifier));
                    }
                    x => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!(
                            "Error in lexing: Found unexpected token {}",
                            x
                        )))
                    }
                },
                None => return Ok(Token::new(Eof)),
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
        Token::new(SemiColon),
        Token::new(Dot),
        Token::new(Plus)
    );

    test_lexer_ok!(
        single_char_tokens,
        ";> = < , { () } +-*/",
        Token::new(SemiColon),
        Token::new(Gt),
        Token::new(Eq),
        Token::new(Lt),
        Token::new(Comma),
        Token::new(LeftBrace),
        Token::new(LeftParen),
        Token::new(RightParen),
        Token::new(RightBrace),
        Token::new(Plus),
        Token::new(Minus),
        Token::new(Star),
        Token::new(ForwardSlash)
    );

    test_lexer_ok!(
        double_char_tokens,
        "== >= <= !=",
        Token::new(Deq),
        Token::new(Ge),
        Token::new(Le),
        Token::new(Ne)
    );

    test_lexer_ok!(
        single_double_char_tokens,
        "==;.((}{))+/.",
        Token::new(Deq),
        Token::new(SemiColon),
        Token::new(Dot),
        Token::new(LeftParen),
        Token::new(LeftParen),
        Token::new(RightBrace),
        Token::new(LeftBrace),
        Token::new(RightParen),
        Token::new(RightParen),
        Token::new(Plus),
        Token::new(ForwardSlash),
        Token::new(Dot)
    );

    test_lexer_ok!(
        ignore_single_line_comment,
        "//Comment to be ignored.\n {}",
        Token::new(LeftBrace),
        Token::new(RightBrace)
    );

    test_lexer_ok!(
        literal_str,
        "\"This is a string followed by a semi-colon.\";",
        Token::new_with_lexeme(Str, "This is a string followed by a semi-colon."),
        Token::new(SemiColon)
    );

    test_lexer_ok!(
        literal_int,
        "12 + 345; ",
        Token::new_with_lexeme(Numeric, "12"),
        Token::new(Plus),
        Token::new_with_lexeme(Numeric, "345"),
        Token::new(SemiColon)
    );

    test_lexer_ok!(
        literal_float,
        "12.123123 + 345 ",
        Token::new_with_lexeme(Numeric, "12.123123"),
        Token::new(Plus),
        Token::new_with_lexeme(Numeric, "345"),
    );

    test_lexer_ok!(
        lex_assignment,
        "a = 52;",
        Token::new_with_lexeme(Ident, "a"),
        Token::new(Eq),
        Token::new_with_lexeme(Numeric, "52"),
        Token::new(SemiColon)
    );

    test_lexer_ok!(
        lex_keywords,
        "if (a=10) { return 1; }",
        Token::new(If),
        Token::new(LeftParen),
        Token::new_with_lexeme(Ident, "a"),
        Token::new(Eq),
        Token::new_with_lexeme(Numeric, "10"),
        Token::new(RightParen),
        Token::new(LeftBrace),
        Token::new(Return),
        Token::new_with_lexeme(Numeric, "1"),
        Token::new(SemiColon),
        Token::new(RightBrace)
    );

    test_lexer_ok!(
        variable_decl,
        "var a;",
        Token::new_with_lexeme(Var, "var"),
        Token::new_with_lexeme(Ident, "a"),
        Token::new(SemiColon)
    );

    test_lexer_ok!(
        logical_op,
        "(1 or 2 and 3)",
        Token::new(LeftParen),
        Token::new_with_lexeme(Numeric, "1"),
        Token::new(Or),
        Token::new_with_lexeme(Numeric, "2"),
        Token::new(And),
        Token::new_with_lexeme(Numeric, "3"),
        Token::new(RightParen)
    );

    test_lexer_err!(
        unterminated_string_literal,
        "\" this string is not terminated",
        JLoxError::UnterminatedStringLiteral
    );
}
