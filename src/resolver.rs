use std::collections::HashMap;
use std::io::Write;

use crate::anyhow;
use crate::ast::*;
use crate::ErrorOrCtxJmp;
use crate::Interpreter;
use crate::Result;
use crate::Stmt;

pub type ResolveResult = Result<()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
    ClassMethod,
    Initializer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'a, W> {
    interpreter: &'a mut Interpreter<W>,
    scopes: Vec<HashMap<&'a str, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a, W: Write> Resolver<'a, W> {
    pub fn new(interpreter: &'a mut Interpreter<W>) -> Self {
        Self {
            interpreter,
            scopes: vec![HashMap::new()],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmt(&mut self, stmt: &'a Stmt) -> ResolveResult {
        match stmt {
            Stmt::Print(e) | Stmt::Expr(e) => self.resolve_expr(e)?,
            Stmt::VariableDecl(VariableDecl { name, definition }) => {
                self.declare(name);
                match definition {
                    Some(initalizer_expr) => {
                        self.resolve_expr(initalizer_expr)?;
                    }
                    None => {}
                }
                self.define(name);
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve(stmts)?;
                self.end_scope();
            }
            Stmt::Conditional(Conditional {
                cond,
                if_branch,
                else_branch,
            }) => {
                self.resolve_expr(cond)?;
                self.resolve_stmt(if_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch)?;
                }
            }
            Stmt::Loop(Loop { cond, body }) => {
                self.resolve_expr(cond)?;
                self.resolve_stmt(body)?;
            }
            Stmt::FunctionDecl(f) => {
                self.declare(&f.name);
                self.define(&f.name);
                self.resolve_function(&f.params, &f.body, FunctionType::Function)?;
            }
            Stmt::Return(expr) => {
                if self.current_function == FunctionType::None {
                    return Err(ErrorOrCtxJmp::Error(anyhow!(
                        "can't return from top level code"
                    )));
                }

                if self.current_function == FunctionType::Initializer {
                    return Err(ErrorOrCtxJmp::Error(anyhow!(
                        "can't return a value from top level code"
                    )));
                }

                self.resolve_expr(expr)?;
            }
            Stmt::ClassDecl(ClassDecl {
                name,
                super_class,
                methods,
            }) => {
                self.declare(name);
                self.define(name);

                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                if let Some(super_class) = super_class {
                    self.resolve_expr(super_class)?;
                    self.begin_scope();
                    self.scopes.last_mut().unwrap().insert("super", true);
                }

                self.begin_scope();
                self.scopes.last_mut().unwrap().insert("this", true);
                for method in methods {
                    let declaration = if method.name.ident == "this" {
                        FunctionType::Initializer
                    } else {
                        FunctionType::ClassMethod
                    };
                    self.resolve_function(&method.params, &method.body, declaration)?;
                }
                self.end_scope();

                if super_class.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
            }
        }
        Ok(())
    }

    pub fn resolve_expr(&mut self, expr: &'a Expr) -> ResolveResult {
        match expr {
            Expr::Nil | Expr::Int(_) | Expr::Float(_) | Expr::Boolean(_) | Expr::String(_) => {}
            Expr::Ident(id) => {
                if !self.scopes.is_empty() {
                    match self.scopes.last().unwrap().get(&id.ident as &str) {
                        Some(b) if !(*b) => {
                            return Err(ErrorOrCtxJmp::Error(anyhow!(
                                "unable to read local variable in its own initalizer"
                            )))
                        }
                        _ => {}
                    };
                }
                self.resolve_local(id)?
            }
            Expr::Unary(_, e) => {
                self.resolve_expr(e)?;
            }
            Expr::Binary(_, e1, e2) | Expr::Logical(_, e1, e2) => {
                self.resolve_expr(e1)?;
                self.resolve_expr(e2)?;
            }
            Expr::Assign(ident, e) => {
                self.resolve_expr(e)?;
                if let Expr::Ident(ref id) = **ident {
                    self.resolve_local(id)?;
                } else {
                    unreachable!()
                };
            }
            Expr::Call(callee, args) => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(&arg.value)?;
                }
            }
            Expr::Lambda(params, body) => {
                self.resolve_function(params, body, FunctionType::Function)?
            }
            Expr::Get(object, _fields) => {
                self.resolve_expr(object)?;
            }
            Expr::Set(object, _, value) => {
                self.resolve_expr(value)?;
                self.resolve_expr(object)?;
            }
            Expr::This(this) => {
                if self.current_class == ClassType::None {
                    return Err(ErrorOrCtxJmp::Error(anyhow!(
                        "can't use 'this' outside class context"
                    )));
                }
                self.resolve_local(this)?
            }
            Expr::Super(super_class, _method) => {
                if self.current_class == ClassType::None {
                    return Err(ErrorOrCtxJmp::Error(anyhow!(
                        "can't use 'super' outside class context"
                    )));
                }
                self.resolve_local(super_class)?;
            }
        }
        Ok(())
    }

    pub fn resolve(&mut self, stmts: &'a [Stmt]) -> ResolveResult {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn resolve_local(&mut self, id: &Identifier) -> ResolveResult {
        for i in (0..self.scopes.len()).rev() {
            let scope = unsafe { self.scopes.get_unchecked(i) };
            match scope.get(&id.ident as &str) {
                Some(_) => {
                    self.interpreter
                        .resolve(id.clone(), self.scopes.len() - 1 - i);
                    return Ok(());
                }
                None => {
                    continue;
                }
            }
        }

        Err(ErrorOrCtxJmp::Error(anyhow!(
            "variable {} not find in any of scope",
            id
        )))
    }

    fn resolve_function(
        &mut self,
        params: &'a [Identifier],
        body: &'a [Stmt],
        ftype: FunctionType,
    ) -> ResolveResult {
        let enclosing_function = self.current_function;
        self.current_function = ftype;
        self.begin_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }

        self.resolve(body)?;

        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &'a Identifier) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(&name.ident, false);
        }
    }

    fn define(&mut self, name: &'a Identifier) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(&name.ident, true);
        }
    }
}
