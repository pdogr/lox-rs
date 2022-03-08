use std::iter::Peekable;

use crate::anyhow;
use crate::ast::*;
use crate::ErrorOrCtxJmp;
use crate::Result;
use crate::Stmt;
use crate::Token;
use crate::TokenType;

type ParseResult = Result<Expr>;
type ParseStmtResult = Result<Stmt>;

pub struct Parser<I: Iterator<Item = Token>> {
    i: Peekable<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(i: I) -> Self {
        Self { i: i.peekable() }
    }

    pub fn next(&mut self) -> Result<Token> {
        match self.i.next() {
            Some(t) => Ok(t),
            None => return Err(ErrorOrCtxJmp::Error(anyhow!("missing token"))),
        }
    }

    fn expect(&mut self, expected: Token, err: &str) -> Result<()> {
        match self.i.peek() {
            Some(actual) if *actual == expected => {
                self.i.next();
                Ok(())
            }
            Some(actual) => {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "expected token {} got token {}, {}",
                    expected,
                    actual,
                    err
                )))
            }

            None => {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "missing token, expected token {}, {}",
                    expected,
                    err
                )))
            }
        }
    }

    fn peek_expect(&mut self, expected: Token) -> bool {
        match self.i.peek() {
            Some(actual) if *actual == expected => true,
            _ => false,
        }
    }

    pub fn program(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while let Some(_tok) = self.i.peek() {
            stmts.push(self.declaration()?);
        }
        Ok(stmts)
    }

    fn declaration(&mut self) -> ParseStmtResult {
        match self.i.peek() {
            Some(TokenType::Var) => self.var_decl(),
            Some(TokenType::Fun) => self.fun_decl(),
            _ => self.statement(),
        }
    }

    fn fun_decl(&mut self) -> ParseStmtResult {
        self.expect(TokenType::Fun, "expected fun as function declaration")?;
        let name = self.identifier()?;

        self.expect(TokenType::LeftParen, "expected ( after function name")?;
        let params = if !self.peek_expect(TokenType::RightParen) {
            self.parameters()?
        } else {
            Vec::new()
        };
        self.expect(TokenType::RightParen, "expected ) after function params")?;
        let body = self.block()?;

        let stmts = if let Stmt::Block(stmts) = body {
            stmts
        } else {
            vec![]
        };

        Ok(Stmt::Function {
            name,
            params,
            body: stmts,
        })
    }

    fn identifier(&mut self) -> Result<String> {
        match self.next()? {
            TokenType::Ident(id) => Ok(id),
            x => {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "expected identifier, got {}",
                    x
                )))
            }
        }
    }

    fn parameters(&mut self) -> Result<Vec<String>> {
        let mut params = vec![self.identifier()?];
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::Comma => {
                    self.next()?;
                    params.push(self.identifier()?);
                }
                _ => break,
            }
        }
        Ok(params)
    }

    fn var_decl(&mut self) -> ParseStmtResult {
        self.expect(TokenType::Var, "expected var keyword in var declaration")?;
        let ident = self.identifier()?;

        let ast = if self.peek_expect(TokenType::Eq) {
            self.next()?;
            let ast = self.expression()?;
            Some(ast)
        } else {
            None
        };

        self.expect(
            TokenType::SemiColon,
            "declaration should be terminated by ;",
        )?;
        Ok(Stmt::Decl {
            ident,
            definition: ast,
        })
    }

    fn statement(&mut self) -> ParseStmtResult {
        match self.i.peek() {
            Some(TokenType::Print) => self.print_stmt(),
            Some(TokenType::LeftBrace) => self.block(),
            Some(TokenType::If) => self.if_stmt(),
            Some(TokenType::Return) => self.return_stmt(),
            Some(TokenType::While) => self.while_stmt(),
            Some(TokenType::For) => self.for_stmt(),
            Some(_) => self.expr_stmt(),
            None => unreachable!(),
        }
    }

    fn expr_stmt(&mut self) -> ParseStmtResult {
        let expr = self.expression()?;
        self.expect(TokenType::SemiColon, "expression should be terminated by ;")?;
        Ok(Stmt::Expr(expr))
    }

    fn for_stmt(&mut self) -> ParseStmtResult {
        self.expect(TokenType::For, "for loop must start with for keyword")?;
        self.expect(TokenType::LeftParen, "expected ( at the start of for loop")?;
        let mut block = Vec::new();

        let initializer = match self.i.peek() {
            Some(TokenType::SemiColon) => {
                self.expect(
                    TokenType::SemiColon,
                    "initalizer in a for loop must be terminated by ;",
                )?;
                None
            }
            Some(TokenType::Var) => Some(self.var_decl()?),
            _ => Some(self.expr_stmt()?),
        };

        if let Some(initializer) = initializer {
            block.push(initializer);
        }

        let cond = if !self.peek_expect(TokenType::SemiColon) {
            self.expression()?
        } else {
            Expr::Boolean(true)
        };

        self.expect(
            TokenType::SemiColon,
            "condition in a for loop must be terminated by ;",
        )?;
        let update = if !self.peek_expect(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(TokenType::RightParen, "expected ) after for loop")?;

        let body = self.statement()?;
        let loop_body = if let Some(update) = update {
            Stmt::Block(if let Stmt::Block(mut stmts) = body {
                stmts.push(Stmt::Expr(update));
                stmts
            } else {
                vec![body, Stmt::Expr(update)]
            })
        } else {
            body
        };
        block.push(Stmt::Loop {
            cond,
            body: Box::new(loop_body),
        });

        Ok(Stmt::Block(block))
    }

    fn if_stmt(&mut self) -> ParseStmtResult {
        self.expect(TokenType::If, "if statement must start with if keyword")?;
        self.expect(
            TokenType::LeftParen,
            "condition in if statement must start with (",
        )?;
        let cond = self.expression()?;
        self.expect(
            TokenType::RightParen,
            "condition in if statement must end with )",
        )?;
        let if_branch = self.statement()?;
        let else_branch = if self.peek_expect(TokenType::Else) {
            self.next()?;
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::Cond {
            cond,
            if_branch: Box::new(if_branch),
            else_branch,
        })
    }

    fn print_stmt(&mut self) -> ParseStmtResult {
        self.expect(
            TokenType::Print,
            "print statement must begin with print keyword",
        )?;
        let expr = self.expression()?;
        self.expect(
            TokenType::SemiColon,
            "expected ; at the end of print statement",
        )?;
        Ok(Stmt::Print(expr))
    }

    fn return_stmt(&mut self) -> ParseStmtResult {
        self.expect(
            TokenType::Return,
            "return statement must begin with return keyword",
        )?;
        let value = if !self.peek_expect(TokenType::SemiColon) {
            self.expression()?
        } else {
            Expr::Nil
        };
        self.expect(
            TokenType::SemiColon,
            "expected ; at the end of return statement",
        )?;
        Ok(Stmt::Return { value })
    }

    fn while_stmt(&mut self) -> ParseStmtResult {
        self.expect(TokenType::While, "while loop must begin with while keyword")?;
        let cond = self.expression()?;
        let body = self.statement()?;
        Ok(Stmt::Loop {
            cond,
            body: Box::new(body),
        })
    }

    fn block(&mut self) -> ParseStmtResult {
        let mut stmts = Vec::new();
        self.expect(
            TokenType::LeftBrace,
            "expected { at the start of an expression block",
        )?;
        while !self.peek_expect(TokenType::RightBrace) {
            stmts.push(self.declaration()?);
        }
        self.expect(
            TokenType::RightBrace,
            "expected } at the end of an expression block",
        )?;
        Ok(Stmt::Block(stmts))
    }

    pub(crate) fn expression(&mut self) -> ParseResult {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult {
        let ast = self.logic_or()?;

        Ok(if self.peek_expect(TokenType::Eq) {
            self.expect(TokenType::Eq, "expected = in variable assignment")?;
            let inner = self.assignment()?;
            Expr::Assign(Box::new(ast), Box::new(inner))
        } else {
            ast
        })
    }

    fn logic_or(&mut self) -> ParseResult {
        let mut ast = self.logic_and()?;
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::Or => {
                    self.next()?;
                    let inner = self.logic_and()?;
                    ast = Expr::Logical(BinaryOp::Or, Box::new(ast), Box::new(inner));
                }
                _ => break,
            }
        }
        Ok(ast)
    }

    fn logic_and(&mut self) -> ParseResult {
        let mut ast = self.equality()?;
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::And => {
                    self.next()?;
                    let inner = self.equality()?;
                    ast = Expr::Logical(BinaryOp::And, Box::new(ast), Box::new(inner));
                }
                _ => break,
            }
        }
        Ok(ast)
    }

    fn equality(&mut self) -> ParseResult {
        let mut ast = self.comparison()?;
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::Ne | TokenType::Deq => {
                    let bop: BinaryOp = tok.into();
                    self.next()?;
                    let inner = self.comparison()?;
                    ast = Expr::Binary(bop, Box::new(ast), Box::new(inner))
                }
                _ => break,
            }
        }
        Ok(ast)
    }

    fn comparison(&mut self) -> ParseResult {
        let mut ast = self.term()?;
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::Lt | TokenType::Gt | TokenType::Le | TokenType::Ge => {
                    let bop: BinaryOp = tok.into();
                    self.next()?;
                    let inner = self.term()?;
                    ast = Expr::Binary(bop, Box::new(ast), Box::new(inner))
                }
                _ => break,
            }
        }
        Ok(ast)
    }

    fn term(&mut self) -> ParseResult {
        let mut ast = self.factor()?;
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::Plus | TokenType::Minus => {
                    let bop: BinaryOp = tok.into();
                    self.next()?;
                    let inner = self.factor()?;
                    ast = Expr::Binary(bop, Box::new(ast), Box::new(inner))
                }
                _ => break,
            }
        }
        Ok(ast)
    }

    fn factor(&mut self) -> ParseResult {
        let mut ast = self.unary()?;
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::Star | TokenType::ForwardSlash => {
                    let bop: BinaryOp = tok.into();
                    self.next()?;
                    let inner = self.unary()?;
                    ast = Expr::Binary(bop, Box::new(ast), Box::new(inner))
                }
                _ => break,
            }
        }
        Ok(ast)
    }

    fn unary(&mut self) -> ParseResult {
        match self.i.peek() {
            Some(tok) if (*tok == TokenType::Not || *tok == TokenType::Minus) => {
                let uop = match *tok {
                    TokenType::Not => UnaryOp::Not,
                    TokenType::Minus => UnaryOp::Minus,
                    _ => unreachable!(),
                };
                self.next()?;
                let ast = self.unary()?;
                Ok(Expr::Unary(uop, Box::new(ast)))
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> ParseResult {
        let mut callee = self.primary()?;
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::LeftParen => {
                    self.next()?;
                    let args = if self.peek_expect(TokenType::RightParen) {
                        Vec::new()
                    } else {
                        self.arguments()?
                    };
                    self.expect(
                        TokenType::RightParen,
                        "expected ) after params in call statement",
                    )?;
                    callee = Expr::Call(Box::new(callee), args);
                }
                _ => break,
            }
        }
        Ok(callee)
    }

    fn arguments(&mut self) -> Result<Vec<Expr>> {
        let mut args = vec![self.expression()?];
        while let Some(tok) = self.i.peek() {
            match *tok {
                TokenType::Comma => {
                    self.next()?;
                    args.push(self.expression()?);
                }
                _ => break,
            }
            if args.len() >= 255 {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "expected lotta arguments, at max 255 supported, got {} args",
                    args.len()
                )));
            }
        }
        Ok(args)
    }

    fn primary(&mut self) -> ParseResult {
        let next = self.next()?;
        Ok(match next {
            TokenType::Str(s) => Expr::String(s),
            TokenType::Numeric(n) => match n.parse::<i64>() {
                Ok(i) => Expr::Int(i),
                Err(_) => match n.parse::<f64>() {
                    Ok(f) => Expr::Float(f),
                    Err(e) => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!(
                            "unable to parse numerical {} as float with error {}",
                            n,
                            e
                        )))
                    }
                },
            },
            TokenType::Nil => Expr::Nil,
            TokenType::True => Expr::Boolean(true),
            TokenType::False => Expr::Boolean(false),
            TokenType::LeftParen => {
                let ast = self.expression()?;
                self.expect(TokenType::RightParen, "expected ) after expression")?;
                ast
            }
            // Lambda function
            TokenType::Fun => {
                self.expect(
                    TokenType::LeftParen,
                    "expected ( before params in anonymous function",
                )?;
                let params = if !self.peek_expect(TokenType::RightParen) {
                    self.parameters()?
                } else {
                    Vec::new()
                };
                self.expect(
                    TokenType::RightParen,
                    "expected ) after params in anonymous function",
                )?;
                let body = self.block()?;

                let stmts = if let Stmt::Block(stmts) = body {
                    stmts
                } else {
                    vec![]
                };

                Expr::Lambda(params, stmts)
            }
            TokenType::Ident(id) => Expr::Ident(id),
            _ => unreachable!(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::Token;

    #[allow(unused_macros)]
    macro_rules! test_parse {
        ($name: ident,$input: literal,$tt: expr) => {
            #[test]
            fn $name() {
                let input = $input;
                let lexer = Lexer::new(input.chars()).unwrap();
                let tokens: Result<Vec<Token>> = lexer.into_iter().collect();
                let tokens = tokens.expect("lexing error");
                let ast = Parser::new(tokens.into_iter())
                    .expression()
                    .expect("parsing error");

                assert_eq!(ast, $tt);
            }
        };
    }

    #[allow(unused_macros)]
    macro_rules! test_fmt {
        ($name: ident,$input: literal,$tt: expr) => {
            #[test]
            fn $name() {
                let input = $input;
                let lexer = Lexer::new(input.chars()).unwrap();
                let tokens: Result<Vec<Token>> = lexer.into_iter().collect();
                let tokens = tokens.expect("lexing error");
                let ast = Parser::new(tokens.into_iter())
                    .expression()
                    .expect("parsing error");

                assert_eq!(ast.to_string(), $tt);
            }
        };
    }

    test_parse!(number, "(42)", Expr::Int(42));
    test_parse!(
        string,
        "((\"this is a string\"))",
        Expr::String("this is a string".into())
    );
    test_parse!(true_expr, "true", Expr::Boolean(true));
    test_parse!(false_expr, "false", Expr::Boolean(false));
    test_parse!(nil, "nil", Expr::Nil);
    test_parse!(
        float_mul,
        "0.1 * 0.2* 0.3",
        Expr::Binary(
            BinaryOp::Mul,
            Box::new(Expr::Binary(
                BinaryOp::Mul,
                Box::new(Expr::Float(0.1),),
                Box::new(Expr::Float(0.2),)
            )),
            Box::new(Expr::Float(0.3))
        )
    );
    test_parse!(
        float_add_mul,
        "0.1 + 0.2* 0.3",
        Expr::Binary(
            BinaryOp::Add,
            Box::new(Expr::Float(0.1)),
            Box::new(Expr::Binary(
                BinaryOp::Mul,
                Box::new(Expr::Float(0.2),),
                Box::new(Expr::Float(0.3),)
            )),
        )
    );

    test_parse!(
        float_not_add_mul,
        "!0.1 + 0.2* 0.3",
        Expr::Binary(
            BinaryOp::Add,
            Box::new(Expr::Unary(UnaryOp::Not, Box::new(Expr::Float(0.1)))),
            Box::new(Expr::Binary(
                BinaryOp::Mul,
                Box::new(Expr::Float(0.2),),
                Box::new(Expr::Float(0.3),)
            )),
        )
    );

    test_parse!(
        float_bracket_negate_add_mul,
        "-(0.1 + 0.2* 0.3)",
        Expr::Unary(
            UnaryOp::Minus,
            Box::new(Expr::Binary(
                BinaryOp::Add,
                Box::new(Expr::Float(0.1)),
                Box::new(Expr::Binary(
                    BinaryOp::Mul,
                    Box::new(Expr::Float(0.2),),
                    Box::new(Expr::Float(0.3),)
                )),
            ))
        )
    );

    test_parse!(
        parse_lambda,
        "fun (a){print a;}",
        Expr::Lambda(vec!["a".into()], vec![Stmt::Print(Expr::Ident("a".into()))])
    );

    test_fmt!(mix_expr, "-(-1+2) >=3", "(>= (- (+ (- 1) 2)) 3)");
    test_fmt!(assign_expr, "a=10+23", "(= a (+ 10 23))");
    test_fmt!(assign_multiple, "a=b=10+c", "(= a (= b (+ 10 c)))");
    test_fmt!(
        logical_expr,
        "(a and b or c or d)",
        "(or (or (and a b) c) d)"
    );
}
