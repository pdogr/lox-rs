use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use lexer::Span;
use lexer::Token;
use lexer::TokenType;

use crate::anyhow;
use crate::ast::*;
use crate::ErrorOrCtxJmp;
use crate::EvalResult;
use crate::Interpreter;
use crate::Result;

pub(crate) trait Arity {
    fn arity(&self) -> Result<usize>;
}

impl Arity for FuncObject {
    #[inline(always)]
    fn arity(&self) -> Result<usize> {
        Ok(self.params.len())
    }
}

impl Arity for ClassObject {
    #[inline(always)]
    fn arity(&self) -> Result<usize> {
        Ok(if let Some(init_method) = self.find_method("init") {
            init_method.arity()?
        } else {
            0
        })
    }
}

impl Arity for Object {
    #[inline(always)]
    fn arity(&self) -> Result<usize> {
        match self {
            Object::Function(f) => f.arity(),
            Object::Class(c) => c.arity(),
            _ => {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "Can only call functions and classes.",
                )));
            }
        }
    }
}

pub(crate) trait Callable<W>: Arity {
    fn call(&self, args: Vec<Object>, ctx: &mut Interpreter<W>) -> EvalResult;
}

impl<W: Write> Callable<W> for FuncObject {
    #[inline(always)]
    fn call(&self, args: Vec<Object>, ctx: &mut Interpreter<W>) -> EvalResult {
        if args.len() != self.params.len() {
            return Err(ErrorOrCtxJmp::Error(anyhow!(
                "Expected {} arguments but got {}.",
                self.params.len(),
                args.len()
            )));
        }

        ctx.save_env(Rc::clone(&self.closure));
        ctx.push_scope();

        for (param, arg) in self
            .params
            .as_ref()
            .clone()
            .into_iter()
            .zip(args.into_iter())
        {
            ctx.env.borrow_mut().init_variable(param, arg);
        }

        let mut function_result = match ctx.run_many(&self.body) {
            Ok(()) => Object::Nil,
            Err(ErrorOrCtxJmp::RetJump { object }) => object,
            e => {
                e?;
                Object::Nil
            }
        };

        if self.is_initializer {
            function_result = get_env(
                &ctx.env.borrow(),
                &Token::new(TokenType::This, Span::default()).into(),
                1,
            )?
            .borrow()
            .clone();
        }

        ctx.pop_scope();
        ctx.reset_env();

        Ok(function_result)
    }
}

impl<W: Write> Callable<W> for ClassObject {
    #[inline(always)]
    fn call(&self, args: Vec<Object>, ctx: &mut Interpreter<W>) -> EvalResult {
        if args.len() != self.arity().unwrap() {
            return Err(ErrorOrCtxJmp::Error(anyhow!(
                "Expected {} arguments but got {}.",
                self.arity().unwrap(),
                args.len()
            )));
        }
        let instance = Rc::new(RefCell::new(ClassInstance::new(self.clone(), vec![])));

        if let Some(init_method) = self.find_method("init") {
            FuncObject::bind(init_method, Rc::clone(&instance))?.call(args, ctx)?;
        }

        Ok(Object::Instance(instance))
    }
}

impl<W: Write> Callable<W> for Object {
    fn call(&self, args: Vec<Object>, ctx: &mut Interpreter<W>) -> EvalResult {
        match self {
            Object::Function(f) => f.call(args, ctx),
            Object::Class(c) => c.call(args, ctx),
            _ => {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "Can only call functions and classes.",
                )));
            }
        }
    }
}
