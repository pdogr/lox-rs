use std::collections::BTreeMap;
use std::io::Write;
use std::rc::Rc;

use crate::anyhow;
use crate::ast::*;
use crate::new_env;
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
    pub(crate) locals: BTreeMap<Identifier, usize>,
}

impl<W: Write> Interpreter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            env: new_env(),
            envs: Vec::new(),
            locals: BTreeMap::new(),
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
            Stmt::VariableDecl(VariableDecl { name, definition }) => {
                let definition = definition.unwrap_or(Expr::Nil);
                let value = Evaluator::evaluate(definition, Rc::clone(&self.env), self)?;
                self.env.borrow_mut().insert(name, value);
            }
            Stmt::Block(stmts) => {
                self.push_scope();
                self.run_many(stmts.to_vec())?;
                self.pop_scope();
            }
            Stmt::Conditional(Conditional {
                cond,
                if_branch,
                else_branch,
            }) => {
                let cond = Evaluator::evaluate(cond, Rc::clone(&self.env), self)?;
                match cond.is_truth() {
                    true => {
                        self.run(*if_branch)?;
                    }
                    false => {
                        if let Some(else_branch) = else_branch {
                            self.run(*else_branch)?;
                        }
                    }
                };
            }
            Stmt::Loop(Loop { cond, body }) => loop {
                let cond_val = Evaluator::evaluate(cond.clone(), Rc::clone(&self.env), self)?;
                if !cond_val.is_truth() {
                    break;
                }
                self.run(*body.clone())?;
            },
            Stmt::FunctionDecl(FunctionDecl { name, params, body }) => {
                let func = Object::Function(FuncObject::new(
                    name.clone(),
                    params,
                    body,
                    self.env.clone(),
                    false,
                ));
                self.env.borrow_mut().insert(name, func);
            }
            Stmt::Return(value) => {
                let value = Evaluator::evaluate(value, Rc::clone(&self.env), self)?;
                return Err(ErrorOrCtxJmp::RetJump { object: value });
            }
            Stmt::ClassDecl(ClassDecl {
                name,
                super_class,
                methods,
            }) => {
                let (super_class, has_super_class) = if let Some(super_class) = super_class {
                    let sc = Evaluator::evaluate(super_class, Rc::clone(&self.env), self)?;
                    match sc {
                        Object::Class(c) => (Some(Box::new(c)), true),
                        _ => {
                            return Err(ErrorOrCtxJmp::Error(anyhow!(
                                "Superclass must be a class."
                            )))
                        }
                    }
                } else {
                    (None, false)
                };

                if let Some(ref sc) = super_class {
                    self.push_scope();
                    let scc = *sc.clone();
                    self.env
                        .borrow_mut()
                        .insert("super".to_string().into(), Object::Class(scc));
                }

                let class = Object::Class(ClassObject::new(
                    name.clone(),
                    super_class,
                    methods
                        .into_iter()
                        .map(|method| {
                            let name = method.name;
                            let is_initializer = &name.ident == "init";
                            (
                                name.ident.clone(),
                                FuncObject::new(
                                    name,
                                    method.params,
                                    method.body,
                                    Rc::clone(&self.env),
                                    is_initializer,
                                ),
                            )
                        })
                        .collect(),
                ));

                if has_super_class {
                    self.pop_scope();
                }
                self.env.borrow_mut().insert(name, class);
            }
        };
        Ok(())
    }

    pub fn resolve(&mut self, id: Identifier, distance: usize) {
        self.locals.insert(id, distance);
    }

    pub fn run_many(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            self.run(stmt)?;
        }
        Ok(())
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
    use crate::Resolver;
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
                    let mut resolver = Resolver::new();
                    resolver
                        .resolve(&stmts, &mut interpreter)
                        .expect("variable resolution error");

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
        "\"global\"\n\"global\"\n"
    );

    test_interpret_ok!(class_declaration, "class Bagel{}", "");

    test_interpret_ok!(
        class_declaration_print,
        r#"
        class DevonshireCream{
            serveOn(){
                return "Scones";
            }
        }
        print DevonshireCream;
        "#,
        "<class DevonshireCream>\n"
    );

    test_interpret_ok!(
        class_constructor,
        r#"
        class Bagel{}
        Bagel();
        "#,
        ""
    );

    test_interpret_ok!(
        class_instance,
        r#"
        class Bagel {}
        var bagel = Bagel();
        print bagel;
        "#,
        "<instance@Bagel>\n"
    );

    test_interpret_ok!(
        instance_setter,
        r#"
        class Bagel {}
        var bagel = Bagel();

        bagel.type = "food";
        bagel.amount = 42;

        print bagel.type;
        print bagel.amount;

        bagel.type = nil;

        print bagel.type;
        "#,
        "\"food\"\n42\nnil\n"
    );

    test_interpret_ok!(
        instance_method,
        r#"
        class Bacon {
            eat(){
                print "Crunch crunch";
            }
        }
        Bacon().eat();
        "#,
        "\"Crunch crunch\"\n"
    );

    test_interpret_ok!(
        print_this,
        r#"
        class Egotist{
            speak(){
                print this;
            }
        }
        var method = Egotist().speak;
        method();
        "#,
        "<instance@Egotist>\n"
    );

    test_interpret_ok!(
        instance_this,
        r#"
        class Cake{
            taste(){
                var adj = "delicious";
                print this.flavor + " cake is " + adj + "!";
            }
        }
        var cake = Cake();
        cake.flavor = "Chocolate";
        cake.taste();
        "#,
        "\"Chocolate cake is delicious!\"\n"
    );

    test_interpret_ok!(
        this_callback,
        r#"
        class Thing{
            getCallback(){
                fun local(){
                    print this;
                }
                return local;
            }
        }
        var cb = Thing().getCallback();
        cb();
        "#,
        "<instance@Thing>\n"
    );

    test_interpret_ok!(
        multiple_this,
        r#"
        class Burger{
            show(){
                print this.size + this.shape;
            }
        }
        var roundBurger = Burger();
        roundBurger.size = "42 ";
        roundBurger.shape = "round";
        roundBurger.show();
        "#,
        "\"42 round\"\n"
    );

    test_interpret_ok!(
        class_init,
        r#"
        class Foo {
            init(){
                print this;
            }
        }
        var foo = Foo();
        print foo.init();
        "#,
        "<instance@Foo>\n<instance@Foo>\n<instance@Foo>\n"
    );

    test_interpret_ok!(
        successful_reinit,
        r#"
        class Foo {
            init(n){
                this.n = n;
            }
        }
        var foo = Foo(42);
        print foo.n;
        foo.n = 69;
        print foo.n;
        foo.init(420);
        print foo.n;
        "#,
        "42\n69\n420\n"
    );

    test_interpret_ok!(
        class_inherit,
        r#"
        class Doughnut{
            cook(){
                print "parent class";
            }
        }
        class Child < Doughnut{}
        Child().cook();
        "#,
        "\"parent class\"\n"
    );

    test_interpret_ok!(
        super_class,
        r#"
        class Doughnut{
            cook(){
                print "super";
            }
        }
        class BostonCream < Doughnut {
            cook() {
                super.cook();
                print "child";
            }
        }
        BostonCream().cook();
        "#,
        "\"super\"\n\"child\"\n"
    );

    test_interpret_ok!(
        multiline_string,
        r#"
var a = "1
2
3";
print a;
        "#,
        "\"1\n2\n3\"\n"
    );
}
