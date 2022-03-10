use std::io::Write;

use crate::anyhow;

use crate::ast::*;
use crate::ErrorOrCtxJmp;
use crate::EvalResult;
use crate::Interpreter;
use crate::Object;
use crate::Result;

pub(crate) trait Arity {
    fn arity(&self) -> Result<usize>;
}

impl Arity for FuncObject {
    fn arity(&self) -> Result<usize> {
        Ok(self.params.len())
    }
}

impl Arity for Object {
    fn arity(&self) -> Result<usize> {
        match self {
            Object::Function(f) => f.arity(),
            _ => {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "expected function got {}",
                    self,
                )))
            }
        }
    }
}

pub(crate) trait Callable<W>: Arity {
    fn call(&self, args: Vec<Object>, ctx: &mut Interpreter<W>) -> EvalResult;
}

impl<W: Write> Callable<W> for FuncObject {
    fn call(&self, args: Vec<Object>, ctx: &mut Interpreter<W>) -> EvalResult {
        if args.len() != self.params.len() {
            return Err(ErrorOrCtxJmp::Error(anyhow!(
                "expected {} arguments, got {} arguments",
                self.params.len(),
                args.len()
            )));
        }

        ctx.save_env(self.closure.clone());
        ctx.push_scope();

        self.params
            .clone()
            .into_iter()
            .zip(args.into_iter())
            .for_each(|(param, arg)| {
                ctx.env.borrow_mut().insert(param, arg);
            });

        let function_result = match ctx.run_many(self.body.clone()) {
            Ok(()) => Object::Nil,
            Err(ErrorOrCtxJmp::RetJump { object }) => object,
            e => {
                e?;
                Object::Nil
            }
        };

        ctx.pop_scope();
        ctx.reset_env();

        Ok(function_result)
    }
}

impl<W: Write> Callable<W> for Object {
    fn call(&self, args: Vec<Object>, ctx: &mut Interpreter<W>) -> EvalResult {
        match self {
            Object::Function(f) => f.call(args, ctx),
            _ => {
                return Err(ErrorOrCtxJmp::Error(anyhow!(
                    "expected function got {}",
                    self
                )));
            }
        }
    }
}
