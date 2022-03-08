use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    Decl {
        ident: String,
        definition: Option<Expr>,
    },
    Block(Vec<Stmt>),
    Cond {
        cond: Expr,
        if_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Loop {
        cond: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return {
        value: Expr,
    },
}
