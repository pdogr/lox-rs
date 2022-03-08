use std::io::Write;
use std::rc::Rc;

use crate::anyhow;
use crate::ast::Expr;
use crate::new_env;
use crate::object::FuncInner;
use crate::pop_env;
use crate::push_env;
use crate::Env;
use crate::ErrorOrCtxJmp;
use crate::Evaluator;
use crate::Object;
use crate::Result;
use crate::Stmt;

#[derive(Debug)]
pub struct Interpreter<W> {
    pub(crate) writer: W,
    pub(crate) env: Env,
    envs: Vec<Env>,
}

impl<W: Write> Interpreter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            env: new_env(),
            envs: Vec::new(),
        }
    }

    fn run(&mut self, stmt: Stmt) -> Result<()> {
        match stmt {
            Stmt::Print(expr) => {
                let o = Evaluator::evaluate(expr, Rc::clone(&self.env), self)?;
                let res = writeln!(self.writer, "{}", o);
                if res.is_err() {
                    return Err(ErrorOrCtxJmp::Error(anyhow!("unable to write")));
                }
            }
            Stmt::Expr(expr) => {
                let _ = Evaluator::evaluate(expr, Rc::clone(&self.env), self)?;
            }
            Stmt::Decl { ident, definition } => {
                let definition = definition.unwrap_or(Expr::Nil);
                let value = Evaluator::evaluate(definition, Rc::clone(&self.env), self)?;
                self.env.borrow_mut().insert(ident, value);
            }
            Stmt::Block(stmts) => {
                self.push_scope();
                self.run_many(stmts.to_vec())?;
                self.pop_scope();
            }
            Stmt::Cond {
                cond,
                if_branch,
                else_branch,
            } => {
                let cond = Evaluator::evaluate(cond, Rc::clone(&self.env), self)?;
                match cond {
                    Object::Boolean(true) => {
                        self.run(*if_branch)?;
                    }
                    Object::Boolean(false) => {
                        if let Some(else_branch) = else_branch {
                            self.run(*else_branch)?;
                        }
                    }
                    x => return Err(ErrorOrCtxJmp::Error(anyhow!("expected bool found {}", x))),
                };
            }
            Stmt::Loop { cond, body } => loop {
                let cond_val = Evaluator::evaluate(cond.clone(), Rc::clone(&self.env), self)?;
                if !cond_val.is_truth() {
                    break;
                }
                self.run(*body.clone())?;
            },
            Stmt::Function { name, params, body } => {
                let func =
                    Object::Func(FuncInner::new(name.clone(), params, body, self.env.clone()));
                self.env.borrow_mut().insert(name, func);
            }
            Stmt::Return { value } => {
                let value = Evaluator::evaluate(value, Rc::clone(&self.env), self)?;
                return Err(ErrorOrCtxJmp::RetJump { object: value });
            }
        };
        Ok(())
    }

    pub fn run_many(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        stmts.into_iter().fold(Ok(()), |res, stmt| {
            if res.is_err() {
                return res;
            }
            self.run(stmt)
        })
    }

    pub(crate) fn save_env(&mut self, env: Env) {
        self.envs.push(Rc::clone(&self.env));
        self.env = env;
    }

    pub(crate) fn reset_env(&mut self) {
        self.env = self
            .envs
            .pop()
            .expect("poping env from empty stack, this is a BUG");
    }

    pub(crate) fn push_scope(&mut self) {
        self.env = push_env(Rc::clone(&self.env));
    }

    pub(crate) fn pop_scope(&mut self) {
        self.env = pop_env(Rc::clone(&self.env));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::test_utils::TestWriter;
    use crate::Token;

    #[allow(unused_macros)]
    macro_rules! test_interpret_ok {
        ($name: ident,$input: literal,$tt: expr) => {
            #[test]
            fn $name() {
                let fake_stdout = TestWriter::new();
                {
                    let input = $input;
                    let lexer = Lexer::new(input.chars()).unwrap();
                    let tokens: Result<Vec<Token>> = lexer.into_iter().collect();
                    let tokens = tokens.expect("lexing error");
                    let stmts = Parser::new(tokens.into_iter())
                        .program()
                        .expect("parsing error");

                    let mut interpreter = Interpreter::new(fake_stdout.clone());
                    interpreter.run_many(stmts).expect("interpret error");
                }
                assert_eq!(&fake_stdout.into_string(), $tt);
            }
        };
    }

    test_interpret_ok!(print_string, r#" print "one"; "#, "\"one\"\n");
    test_interpret_ok!(
        print_multiple,
        r#" print "one"; print true; print 20+22; "#,
        "\"one\"\ntrue\n42\n"
    );
    test_interpret_ok!(var_decl, r#" var a = 1; var b =2; print a+b;"#, "3\n");
    test_interpret_ok!(
        var_assign,
        "var a; a=10; print a*a; a=-1; print a+a;",
        "100\n-2\n"
    );
    test_interpret_ok!(print_var_assign, "var a; print a=2;", "2\n");
    test_interpret_ok!(new_scope, "var a=10;print a;{ a=11;print a; }", "10\n11\n");
    test_interpret_ok!(
        multi_scope,
        r#" 
        var a = "global a";
        var b = "global b";
        var c = "global c";
        {
            var a = "outer a";
            var b = "outer b";
            {
                var a = "inner a";
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
            }
        print a;
        print b;
        print c;
        "#,
        r#""inner a"
"outer b"
"global c"
"outer a"
"outer b"
"global c"
"global a"
"global b"
"global c"
"#
    );
    test_interpret_ok!(
        if_stmt,
        r#" print 10; if(true) { print true; } else {print false;}"#,
        "10\ntrue\n"
    );

    test_interpret_ok!(
        or_short_circuit,
        r#"var a= 10; var b=false; if( a or b ) {print true;} else {print false;}"#,
        "true\n"
    );
    test_interpret_ok!(
        and_short_circuit,
        r#"var a= 10; var b=false; if( a and b ) {print true;} else {print false;}"#,
        "false\n"
    );
    /*
    test_interpret_ok!(
        while_stmt,
        r#" var i=1; var sum=0; while (i<10) {i=i+1; sum=sum+i;} print sum;"#,
        "45\n"
    );
    */
    test_interpret_ok!(
        variable_add,
        r#" var i=100; { i=i+111; } print i;"#,
        "211\n"
    );

    test_interpret_ok!(
        for_all_opts,
        r#"
        var sum=1;
        for(var i=1;i<=10;i=i+1){
            sum=sum*i;
        }
        print sum;
        "#,
        "3628800\n"
    );

    test_interpret_ok!(
        for_without_init,
        r#"
        var i=0;
        var sum=0;
        for(;i<100;i=i+1){
            sum = sum+i*69;
        }
        print sum;
        "#,
        "341550\n"
    );

    test_interpret_ok!(
        for_loop_fibs,
        r#"
        var a=1;
        var temp;
        for(var b=1; a<10000;b=temp+b){
            print a;
            temp=a;
            a=b;
        }
        "#,
        "1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n89\n144\n233\n377\n610\n987\n1597\n2584\n4181\n6765\n"
    );

    test_interpret_ok!(
        fn_print_num,
        r#"
        fun printNum(n) {
            print n;
        }
        printNum(10);
        "#,
        "10\n"
    );

    test_interpret_ok!(
        fn_return_stmt,
        r#"
        fun returnInt(n){
            return n*100;
        }
        print returnInt(42);
        "#,
        "4200\n"
    );

    test_interpret_ok!(
        fn_count,
        r#"
        fun count(n){
            while(n<20){
                if (n==5) return n;
                print n;
                n = n+1;
            }
        }

        count(1);
        "#,
        "1\n2\n3\n4\n"
    );

    test_interpret_ok!(
        fn_add,
        r#"
        fun add(a,b,c) {
            print a+b+c;
        }
        add(1,2,3);
        "#,
        "6\n"
    );

    test_interpret_ok!(
        fn_recursive_fib,
        r#"
        fun fib(n){
            if (n<=1) return n;
            return fib(n-2)+fib(n-1);
        }
        for (var i=0;i<20;i=i+1){
            print fib(i);
        }
        "#,
        "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n89\n144\n233\n377\n610\n987\n1597\n2584\n4181\n"
    );

    test_interpret_ok!(
        fn_inner_fn,
        r#"
        fun makeCounter(){
            var i=0;
            fun count(){
                i = i + 1;
                print i;
            }
            return count;
        }

        var counter = makeCounter();
        counter();
        counter();
        "#,
        "1\n2\n"
    );

    test_interpret_ok!(
        fn_as_arg,
        r#"
        fun thrice(fn) {
            for (var i = 1; i <= 3; i = i + 1) {
                fn(i);
            }
        }
        thrice(
            fun (a) {
                print a;
            }
        );
        "#,
        "1\n2\n3\n"
    );

    /*
        test_interpret_ok!(
            closure_scope,
            r#"
            var a = "global";
            {
                fun showA(){
                    print a;
                }
                showA();
                var a = "block";
                showA();
            }
            "#,
            r#"
    global
    global"#
        );
        */
}
