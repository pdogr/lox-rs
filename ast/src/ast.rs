use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

use lox_lexer::Token;

use crate::push_env;
use crate::Env;
use crate::EnvErrorKind;
use crate::Result;
use crate::Span;
use crate::TokenType;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum UnaryOp {
    Minus,
    Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            UnaryOp::Minus => "-",
            UnaryOp::Not => "!",
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum BinaryOp {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Lt,  // <
    Gt,  // >
    Eq,  // ==
    Le,  // <=
    Ge,  // >=
    Ne,  // !=
    Or,  // ||
    And, // &&
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Eq => "==",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::Ne => "!=",
            BinaryOp::Or => "or",
            BinaryOp::And => "and",
        };
        write!(f, "{}", c)
    }
}

impl From<TokenType> for BinaryOp {
    fn from(ttype: TokenType) -> Self {
        use TokenType::*;
        match ttype {
            Plus => BinaryOp::Add,
            Minus => BinaryOp::Sub,
            Star => BinaryOp::Mul,
            ForwardSlash => BinaryOp::Div,
            Lt => BinaryOp::Lt,
            Gt => BinaryOp::Gt,
            Le => BinaryOp::Le,
            Ge => BinaryOp::Ge,
            Deq => BinaryOp::Eq,
            Ne => BinaryOp::Ne,
            Or => BinaryOp::Or,
            And => BinaryOp::And,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub token: Token,
    pub rid: usize,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl From<Token> for Identifier {
    fn from(token: Token) -> Self {
        Self { token, rid: 0 }
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl Eq for Identifier {}

impl Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.token.span.line.hash(state);
        self.token.span.col.hash(state);
        self.token.lexeme.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub value: Expr,
}

impl From<Expr> for Argument {
    fn from(e: Expr) -> Self {
        Self { value: e }
    }
}

impl From<Argument> for Expr {
    fn from(a: Argument) -> Self {
        a.value
    }
}

pub type Arguments = Vec<Argument>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Int(i64),
    Float(f64),
    Boolean(bool),
    Ident(Identifier),
    String(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Logical(BinaryOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Arguments),
    Lambda(Vec<Identifier>, Vec<Stmt>),
    Get(Box<Expr>, Identifier),
    Set(Box<Expr>, Identifier, Box<Expr>),
    This(Identifier),
    Super(Identifier, Identifier),
}

impl Eq for Expr {}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub name: Identifier,
    pub definition: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: Identifier,
    pub params: Vec<Identifier>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub cond: Expr,
    pub if_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    pub cond: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: Identifier,
    pub super_class: Option<Expr>,
    pub methods: Vec<FunctionDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    VariableDecl(VariableDecl),
    Block(Vec<Stmt>),
    Conditional(Conditional),
    Loop(Loop),
    FunctionDecl(FunctionDecl),
    Return(Expr),
    ClassDecl(ClassDecl),
}

#[derive(Clone)]
pub struct FuncObject {
    pub name: Option<Identifier>,
    pub params: Rc<Vec<Identifier>>,
    pub body: Rc<Vec<Stmt>>,
    pub closure: Env,
    pub is_initializer: bool,
}

impl FuncObject {
    #[inline(always)]
    pub fn new(
        name: Identifier,
        params: Vec<Identifier>,
        body: Vec<Stmt>,
        closure: Env,
        is_initializer: bool,
    ) -> Self {
        Self {
            name: Some(name),
            params: Rc::new(params),
            body: Rc::new(body),
            closure,
            is_initializer,
        }
    }

    #[inline(always)]
    pub fn new_lambda(params: Vec<Identifier>, body: Vec<Stmt>, closure: Env) -> Self {
        Self {
            name: None,
            params: Rc::new(params),
            body: Rc::new(body),
            closure,
            is_initializer: false,
        }
    }

    #[inline(always)]
    pub fn bind(f: FuncObject, instance: Rc<RefCell<ClassInstance>>) -> Result<Self> {
        let env = push_env(f.closure);
        env.borrow_mut().init_variable(
            Token::new(TokenType::This, Span::default()).into(),
            Object::Instance(instance),
        );
        Ok(Self { closure: env, ..f })
    }
}

impl PartialEq for FuncObject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.params == other.params && self.body == other.body
    }
}

impl Debug for FuncObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuncInner")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("body", &self.body)
            .finish()
    }
}

impl Display for FuncObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name {
            Some(ref name) => write!(f, "<fn {}>", name),
            None => write!(f, "<closure>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassObject {
    pub name: Identifier,
    pub super_class: Option<Box<ClassObject>>,
    pub methods: HashMap<String, FuncObject>,
}

impl Display for ClassObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

impl ClassObject {
    #[inline(always)]
    pub fn new(
        name: Identifier,
        super_class: Option<Box<ClassObject>>,
        methods: Vec<(String, FuncObject)>,
    ) -> Self {
        Self {
            name,
            super_class,
            methods: methods.into_iter().map(|(id, f)| (id, f)).collect(),
        }
    }

    #[inline(always)]
    pub fn find_method(&self, property: &str) -> Option<FuncObject> {
        if let elt @ Some(_) = self.methods.get(property) {
            return elt.cloned();
        }

        if let Some(ref super_class) = self.super_class {
            super_class.find_method(property)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassInstance {
    class: ClassObject,
    fields: HashMap<String, Object>,
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<instance@{}>", self.class.name)
    }
}

impl ClassInstance {
    #[inline(always)]
    pub fn new_empty(class: ClassObject) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn new(class: ClassObject, fields: Vec<(Identifier, Object)>) -> Self {
        Self {
            class,
            fields: fields
                .into_iter()
                .map(|(id, o)| (id.token.lexeme, o))
                .collect(),
        }
    }

    #[inline(always)]
    pub fn get(property: &str, instance: Rc<RefCell<ClassInstance>>) -> Result<Object> {
        if let Some(o) = instance.borrow().fields.get(property) {
            return Ok(o.clone());
        }

        if let Some(m) = instance.borrow().class.find_method(property) {
            return Ok(Object::Function(FuncObject::bind(m, Rc::clone(&instance))?));
        }

        Err(EnvErrorKind::UndefinedProperty(property.into()))
    }

    #[inline(always)]
    pub fn set(&mut self, property: String, value: Object) {
        self.fields.insert(property, value);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Nil,
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Function(FuncObject),
    Class(ClassObject),
    Instance(Rc<RefCell<ClassInstance>>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Int(i) => write!(f, "{}", *i),
            Object::Float(fl) => write!(f, "{}", *fl),
            Object::Boolean(b) => write!(f, "{}", *b),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Function(fo) => write!(f, "{}", fo),
            Object::Class(co) => write!(f, "{}", co),
            Object::Instance(ci) => write!(f, "{}", ci.borrow()),
        }
    }
}

impl Object {
    #[inline(always)]
    pub fn is_truth(&self) -> bool {
        use Object::*;
        !matches!(self, Nil | Boolean(false))
    }
}
