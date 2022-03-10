use std::io::Write;
use std::rc::Rc;

use crate::anyhow;
use crate::ast::*;
use crate::callable::Callable;
use crate::Env;
use crate::ErrorOrCtxJmp;
use crate::Interpreter;
use crate::Object;
use crate::Result;

pub type EvalResult = Result<Object>;

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate<W: Write>(
        expr: Expr,
        env: Env,
        interpreter: &mut Interpreter<W>,
    ) -> EvalResult {
        use BinaryOp::*;
        use Object::*;
        use UnaryOp::*;
        let r = match expr {
            Expr::Nil => Object::Nil,
            Expr::Int(i) => Object::Int(i),
            Expr::Float(f) => Object::Float(f),
            Expr::Boolean(b) => Object::Boolean(b),
            Expr::String(s) => Object::String(s),
            Expr::Ident(i) => env.borrow().get(&i)?.borrow().clone(),
            Expr::Unary(uop, expr) => {
                match (
                    uop,
                    Evaluator::evaluate(*expr, Rc::clone(&env), interpreter)?,
                ) {
                    (Minus, Int(i)) => Int(-i),
                    (Minus, Float(f)) => Float(-f),
                    (Minus, object) => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!(
                            "- not implemented for {}",
                            object
                        )))
                    }

                    (Not, object) => Boolean(!object.is_truth()),
                }
            }
            Expr::Binary(bop, e1, e2) => {
                match (
                    bop,
                    Evaluator::evaluate(*e1, Rc::clone(&env), interpreter)?,
                    Evaluator::evaluate(*e2, Rc::clone(&env), interpreter)?,
                ) {
                    (Add, String(a), String(b)) => String(a + &b),
                    (Add, Int(a), Int(b)) => Int(a + b),
                    (Add, Int(a), Float(b)) => Float(a as f64 + b),
                    (Sub, Int(a), Int(b)) => Int(a - b),
                    (Sub, Int(a), Float(b)) => Float(a as f64 - b),
                    (Mul, Int(a), Int(b)) => Int(a * b),
                    (Mul, Int(a), Float(b)) => Float(a as f64 * b),
                    (Div, Int(a), Int(b)) => Int(a / b),
                    (Div, Int(a), Float(b)) => Float(a as f64 / b),
                    (Add, Float(a), Int(b)) => Float(a + b as f64),
                    (Add, Float(a), Float(b)) => Float(a + b),
                    (Sub, Float(a), Int(b)) => Float(a - b as f64),
                    (Sub, Float(a), Float(b)) => Float(a - b),
                    (Mul, Float(a), Int(b)) => Float(a * b as f64),
                    (Mul, Float(a), Float(b)) => Float(a * b),
                    (Div, Float(a), Int(b)) => Float(a / b as f64),
                    (Div, Float(a), Float(b)) => Float(a / b),
                    (Lt, Int(a), Int(b)) => Boolean(a < b),
                    (Gt, Int(a), Int(b)) => Boolean(a > b),
                    (Le, Int(a), Int(b)) => Boolean(a <= b),
                    (Ge, Int(a), Int(b)) => Boolean(a >= b),
                    (Lt, Float(a), Float(b)) => Boolean(a < b),
                    (Gt, Float(a), Float(b)) => Boolean(a > b),
                    (Le, Float(a), Float(b)) => Boolean(a <= b),
                    (Ge, Float(a), Float(b)) => Boolean(a >= b),
                    (Eq, a, b) => Boolean(a == b),
                    (Ne, a, b) => Boolean(a != b),
                    (bop, o1, o2) => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!(
                            "unexpected binary operation {} with operands {}, {}",
                            bop,
                            o1,
                            o2
                        )))
                    }
                }
            }
            Expr::Assign(ident, e) => {
                let ident = if let Expr::Ident(ident) = *ident {
                    ident
                } else {
                    unreachable!()
                };
                let old = env.borrow_mut().get(&ident)?;
                let value = Evaluator::evaluate(*e, Rc::clone(&env), interpreter)?;
                *old.borrow_mut() = value.clone();
                value
            }
            Expr::Logical(lop, e1, e2) => match lop {
                BinaryOp::And => {
                    let value = Evaluator::evaluate(*e1, Rc::clone(&env), interpreter)?;
                    if !value.is_truth() {
                        Object::Boolean(false)
                    } else {
                        Object::Boolean(
                            Evaluator::evaluate(*e2, Rc::clone(&env), interpreter)?.is_truth(),
                        )
                    }
                }
                BinaryOp::Or => {
                    let value = Evaluator::evaluate(*e1, Rc::clone(&env), interpreter)?;
                    if value.is_truth() {
                        Object::Boolean(true)
                    } else {
                        Object::Boolean(
                            Evaluator::evaluate(*e2, Rc::clone(&env), interpreter)?.is_truth(),
                        )
                    }
                }
                _ => unreachable!(),
            },
            Expr::Call(callee, args) => {
                let callee = Evaluator::evaluate(*callee, Rc::clone(&env), interpreter)?;
                let evaluated_args: Vec<Object> = args
                    .into_iter()
                    .map(|arg| Evaluator::evaluate(arg.into(), Rc::clone(&env), interpreter))
                    .collect::<Result<Vec<_>>>()?;
                callee.call(evaluated_args, interpreter)?
            }
            Expr::Lambda(params, body) => Object::Function(crate::FuncObject::new_lambda(
                params,
                body,
                interpreter.env.clone(),
            )),
        };
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::new_env;
    use crate::parser::Parser;
    use crate::test_utils::TestWriter;
    use crate::Token;

    #[allow(unused_macros)]
    macro_rules! test_eval_expr_ok {
        ($name: ident,$input: literal,$tt: expr) => {
            #[test]
            fn $name() {
                let fake_stdout = TestWriter::new();
                {
                    let mut interpreter = Interpreter::new(fake_stdout.clone());
                    let env = new_env();
                    let input = $input;
                    let lexer = Lexer::new(input.chars()).unwrap();
                    let tokens: Result<Vec<Token>> = lexer.into_iter().collect();
                    let tokens = tokens.expect("lexing error");
                    let ast = Parser::new(tokens.into_iter())
                        .expression()
                        .expect("parsing error");
                    assert_eq!(
                        Evaluator::evaluate(ast, env, &mut interpreter).unwrap(),
                        $tt
                    );
                }
            }
        };
    }

    test_eval_expr_ok!(add_ints, "22 +20", Object::Int(42));
    test_eval_expr_ok!(add_float_int, "22.22 + 11", Object::Float(22.22 + 11.0));
    test_eval_expr_ok!(sub_ints_neg, "100-450", Object::Int(-350));
    test_eval_expr_ok!(not_bool, "!false", Object::Boolean(true));
    test_eval_expr_ok!(mul_neg_ints, "-20*-20", Object::Int(400));

    test_eval_expr_ok!(
        add_strs,
        r#" "con"+ "catenate""#,
        Object::String("concatenate".into())
    );
}
