use std::io::Write;
use std::rc::Rc;

use lexer::Span;
use lexer::Token;
use lexer::TokenType;

use crate::anyhow;
use crate::ast::*;
use crate::callable::Callable;
use crate::ErrorOrCtxJmp;
use crate::Interpreter;
use crate::Result;

pub type EvalResult = Result<Object>;

pub struct Evaluator;

impl Evaluator {
    #[inline(always)]
    pub fn evaluate<W: Write>(
        expr: &Expr,
        env: Env,
        interpreter: &mut Interpreter<W>,
    ) -> EvalResult {
        use BinaryOp::*;
        use Object::*;
        use UnaryOp::*;
        let r = match expr {
            Expr::Nil => Object::Nil,
            Expr::Int(i) => Object::Int(*i),
            Expr::Float(f) => Object::Float(*f),
            Expr::Boolean(b) => Object::Boolean(*b),
            Expr::String(s) => Object::String(s.clone()),
            Expr::Ident(ident) | Expr::This(ident) => {
                let distance = interpreter.get_distance(ident);
                get_env(&env.borrow(), ident, distance)?.borrow().clone()
            }
            Expr::Unary(uop, expr) => match (uop, Evaluator::evaluate(expr, env, interpreter)?) {
                (Minus, Int(i)) => Int(-i),
                (Minus, Float(f)) => Float(-f),
                (Not, object) => Boolean(!object.is_truth()),
                (Minus, _) => {
                    return Err(ErrorOrCtxJmp::Error(anyhow!("Operand must be a number.")));
                }
            },
            Expr::Binary(bop, e1, e2) => {
                match (
                    bop,
                    Evaluator::evaluate(e1, env.clone(), interpreter)?,
                    Evaluator::evaluate(e2, env, interpreter)?,
                ) {
                    (Add, String(a), String(b)) => String(a + &b),
                    (Add, Int(a), Int(b)) => Int(a + b),
                    (Add, Int(a), Float(b)) => Float(a as f64 + b),
                    (Sub, Int(a), Int(b)) => Int(a - b),
                    (Sub, Int(a), Float(b)) => Float(a as f64 - b),
                    (Mul, Int(a), Int(b)) => Int(a * b),
                    (Mul, Int(a), Float(b)) => Float(a as f64 * b),
                    (Div, Float(_) | Int(_), Int(0)) => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!("Cannot divide by 0.",)))
                    }
                    (Div, Float(_) | Int(_), Float(f)) if f == 0.0 => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!("Cannot divide by 0.",)))
                    }
                    (Div, Int(a), Int(b)) => Int(a / b),

                    (Div, Int(a), Float(b)) => Float(a as f64 / b),
                    (Add, Float(a), Int(b)) => Float(a + b as f64),
                    (Add, Float(a), Float(b)) => Float(a + b),
                    (Add, _, _) => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!(
                            "Operands must be two numbers or two strings."
                        )))
                    }
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
                    (Sub | Mul | Div | Lt | Gt | Le | Ge, _, _) => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!("Operands must be numbers.")));
                    }
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
                let ident = if let Expr::Ident(ref ident) = **ident {
                    ident
                } else {
                    unreachable!()
                };
                let distance = interpreter.get_distance(ident);
                let value = Evaluator::evaluate(e, Rc::clone(&env), interpreter)?;
                assign_env(&env.borrow(), ident, distance, value.clone())?;
                value
            }
            Expr::Logical(lop, e1, e2) => match lop {
                BinaryOp::And => {
                    let value = Evaluator::evaluate(e1, Rc::clone(&env), interpreter)?;
                    if !value.is_truth() {
                        value
                    } else {
                        Evaluator::evaluate(e2, Rc::clone(&env), interpreter)?
                    }
                }
                BinaryOp::Or => {
                    let value = Evaluator::evaluate(e1, Rc::clone(&env), interpreter)?;
                    if value.is_truth() {
                        value
                    } else {
                        Evaluator::evaluate(e2, Rc::clone(&env), interpreter)?
                    }
                }
                _ => unreachable!(),
            },
            Expr::Call(callee, args) => {
                let evaluated_args: Vec<Object> = args
                    .iter()
                    .map(|arg| Evaluator::evaluate(&arg.value, Rc::clone(&env), interpreter))
                    .collect::<Result<Vec<_>>>()?;
                let callee = Evaluator::evaluate(callee, env, interpreter)?;
                callee.call(evaluated_args, interpreter)?
            }
            Expr::Lambda(params, body) => Object::Function(ast::FuncObject::new_lambda(
                params.clone(),
                body.clone(),
                interpreter.env.clone(),
            )),
            Expr::Get(object, property) => match Evaluator::evaluate(object, env, interpreter)? {
                Instance(i) => ClassInstance::get(&property.token.lexeme, i)?,
                _ => {
                    return Err(ErrorOrCtxJmp::Error(anyhow!(
                        "Only instances have properties."
                    )))
                }
            },
            Expr::Set(object, property, value) => {
                match Evaluator::evaluate(object, Rc::clone(&env), interpreter)? {
                    Instance(i) => {
                        let value = Evaluator::evaluate(value, env, interpreter)?;
                        i.borrow_mut()
                            .set(property.token.lexeme.clone(), value.clone());
                        value
                    }
                    _ => return Err(ErrorOrCtxJmp::Error(anyhow!("Only instances have fields."))),
                }
            }
            Expr::Super(super_class, method) => {
                let distance = interpreter.get_distance(super_class);
                let super_class = match get_env(&env.borrow(), super_class, distance)?
                    .borrow()
                    .clone()
                {
                    Class(c) => c,
                    _ => unreachable!(),
                };

                let object = match get_env(
                    &env.borrow(),
                    &Token::new(TokenType::This, Span::default()).into(),
                    distance - 1,
                )?
                .borrow()
                .clone()
                {
                    Instance(i) => i,
                    _ => unreachable!(),
                };

                let super_class_method = match super_class.find_method(&method.token.lexeme as &str)
                {
                    Some(m) => m,
                    None => {
                        return Err(ErrorOrCtxJmp::Error(anyhow!(
                            "Undefined property '{}'.",
                            &method.token.lexeme
                        )));
                    }
                };

                Object::Function(FuncObject::bind(super_class_method, object)?)
            }
        };
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::new_env;
    use crate::lexer::Lexer;
    use crate::lexer::Token;
    use crate::parser::Parser;
    use crate::test_utils::TestWriter;

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
                    let tokens: std::result::Result<Vec<Token>, _> = lexer.into_iter().collect();
                    let tokens = tokens.expect("lexing error");
                    let ast = Parser::new(tokens.into_iter())
                        .expression()
                        .expect("parsing error");
                    assert_eq!(
                        Evaluator::evaluate(&ast, env, &mut interpreter).unwrap(),
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
