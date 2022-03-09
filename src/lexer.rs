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
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType::*;
        loop {
            match self.input.next() {
                Some(c) => match c {
                    '(' => return Some(Ok(LeftParen)),
                    ')' => return Some(Ok(RightParen)),
                    '{' => return Some(Ok(LeftBrace)),
                    '}' => return Some(Ok(RightBrace)),
                    '.' => return Some(Ok(Dot)),
                    ',' => return Some(Ok(Comma)),
                    '+' => return Some(Ok(Plus)),
                    '-' => return Some(Ok(Minus)),
                    ';' => return Some(Ok(SemiColon)),
                    '*' => return Some(Ok(Star)),
                    '/' => match self.match_next('/') {
                        true => {
                            self.skip_while(|c| c != '\n');
                            continue;
                        }
                        false => return Some(Ok(ForwardSlash)),
                    },
                    '!' => {
                        return match self.match_next('=') {
                            true => Some(Ok(Ne)),
                            false => Some(Ok(Not)),
                        }
                    }
                    '=' => {
                        return match self.match_next('=') {
                            true => Some(Ok(Deq)),
                            false => Some(Ok(Eq)),
                        }
                    }
                    '<' => {
                        return match self.match_next('=') {
                            true => Some(Ok(Le)),
                            false => Some(Ok(Lt)),
                        }
                    }
                    '>' => {
                        return match self.match_next('=') {
                            true => Some(Ok(Ge)),
                            false => Some(Ok(Gt)),
                        }
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
                            return Some(Err(ErrorOrCtxJmp::Error(anyhow!(
                                "string literal unterminated"
                            ))));
                        }
                        self.skip(1);
                        return Some(Ok(Str(literal)));
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
                        return Some(Ok(Numeric(numeric)));
                    }
                    a if a.is_ascii_alphanumeric() => {
                        let mut identifier = vec![a];
                        identifier.extend(self.take_while(|c| c.is_ascii_alphanumeric()));
                        let identifier = identifier.into_iter().collect();
                        return Some(Ok(KEYWORDS
                            .get(&identifier as &str)
                            .map_or_else(|| Ident(identifier), |ttype| ttype.clone())));
                    }
                    _ => return None,
                },
                None => return None,
            };
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
        SemiColon,
        Dot,
        Plus
    );

    test_lexer_ok!(
        single_char_tokens,
        ";> = < , { () } +-*/",
        SemiColon,
        Gt,
        Eq,
        Lt,
        Comma,
        LeftBrace,
        LeftParen,
        RightParen,
        RightBrace,
        Plus,
        Minus,
        Star,
        ForwardSlash,
    );

    test_lexer_ok!(double_char_tokens, "== >= <= !=", Deq, Ge, Le, Ne,);

    test_lexer_ok!(
        single_double_char_tokens,
        "==;.((}{))+/.",
        Deq,
        SemiColon,
        Dot,
        LeftParen,
        LeftParen,
        RightBrace,
        LeftBrace,
        RightParen,
        RightParen,
        Plus,
        ForwardSlash,
        Dot,
    );

    test_lexer_ok!(
        ignore_single_line_comment,
        "//Comment to be ignored.\n {}",
        LeftBrace,
        RightBrace,
    );

    test_lexer_ok!(
        literal_str,
        "\"This is a string followed by a semi-colon.\";",
        Str("This is a string followed by a semi-colon.".to_string()),
        SemiColon,
    );

    test_lexer_ok!(
        literal_int,
        "12 + 345; ",
        Numeric("12".into()),
        Plus,
        Numeric("345".into()),
        SemiColon,
    );

    test_lexer_ok!(
        literal_float,
        "12.123123 + 345 ",
        Numeric("12.123123".into()),
        Plus,
        Numeric("345".into()),
    );

    test_lexer_ok!(
        lex_assignment,
        "a = 52;",
        Ident("a".into()),
        Eq,
        Numeric("52".into()),
        SemiColon,
    );

    test_lexer_ok!(
        lex_keywords,
        "if (a=10) { return 1; }",
        If,
        LeftParen,
        Ident("a".into()),
        Eq,
        Numeric("10".into()),
        RightParen,
        LeftBrace,
        Return,
        Numeric("1".into()),
        SemiColon,
        RightBrace,
    );

    test_lexer_ok!(variable_decl, "var a;", Var, Ident("a".into()), SemiColon,);

    test_lexer_ok!(
        logical_op,
        "(1 or 2 and 3)",
        LeftParen,
        Numeric("1".into()),
        Or,
        Numeric("2".into()),
        And,
        Numeric("3".into()),
        RightParen
    );

    test_lexer_err!(
        unterminated_string_literal,
        "\" this string is not terminated",
        JLoxError::UnterminatedStringLiteral
    );
}
