use serde::Serialize;

mod debruijn;
mod iter;

#[derive(Clone, Debug, Serialize)]
pub struct Module {
    pub decls: Vec<Decl>,
}

#[derive(Clone, Debug, Serialize)]
pub enum Decl {
    Type(TypeDecl),
    Func(FuncDecl),
}

#[derive(Clone, Debug, Serialize)]
pub struct TypeDecl {
    pub name: TypeVar,
    pub params: Vec<TypeVar>,
    pub body: Type,
}

#[derive(Clone, Debug, Serialize)]
pub struct FuncDecl {
    pub name: ExprVar,
    pub type_params: Vec<TypeVar>,
    pub expr_params: Vec<(ExprVar, Type)>,
    pub return_type: Type,
    pub body: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TypeVar(String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExprVar(String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExprCon(String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum Type {
    Var(TypeVar),
    Syn(TypeVar),
    Int,
    Bool,
    Fun(Vec<Type>, Box<Type>),
    App(Box<Type>, Vec<Type>),
    Forall(Vec<TypeVar>, Box<Type>),
    Record(Vec<(ExprVar, Type)>),
    Variant(Vec<(ExprCon, Option<Type>)>),
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub enum Expr {
    Var(ExprVar),
    Num(i64),
    Bool(bool),
    Lam(Vec<(ExprVar, Option<Type>)>, Box<Expr>),
    App(Box<Expr>, Vec<Expr>),
    BinOp(Box<Expr>, OpCode, Box<Expr>),
    TypeAbs(Vec<TypeVar>, Box<Expr>),
    TypeApp(Box<Expr>, Vec<Type>),
    Let(ExprVar, Option<Type>, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Record(Vec<(ExprVar, Expr)>),
    Proj(Box<Expr>, ExprVar),
    Variant(ExprCon, Option<Box<Expr>>),
    Match(Box<Expr>, Vec<Branch>),
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct Branch {
    pub con: ExprCon,
    pub var: Option<ExprVar>,
    pub rhs: Expr,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum OpCode {
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

impl Type {
    pub fn var_app(var: TypeVar, args: Vec<Self>) -> Self {
        use Type::*;
        App(Box::new(Var(var)), args)
    }
}

impl TypeVar {
    pub fn new(x: &str) -> Self {
        Self(x.to_owned())
    }
}

impl ExprVar {
    pub fn new(x: &str) -> Self {
        Self(x.to_owned())
    }
}

impl ExprCon {
    pub fn new(x: &str) -> Self {
        Self(x.to_owned())
    }
}
