use serde::Serialize;
use std::fmt;

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

#[derive(Clone, Eq, Hash, PartialEq, Serialize)]
pub struct TypeVar(String);

#[derive(Clone, Eq, Hash, PartialEq, Serialize)]
pub struct ExprVar(String);

#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct ExprCon(String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum Type {
    Error,
    Var(TypeVar),
    SynApp(TypeVar, Vec<Type>),
    Int,
    Bool,
    Fun(Vec<Type>, Box<Type>),
    Record(Vec<(ExprVar, Type)>),
    Variant(Vec<(ExprCon, Type)>),
}

#[derive(Clone, Debug, Serialize)]
pub struct TypeScheme {
    pub params: Vec<TypeVar>,
    pub body: Type,
}

#[derive(Clone, Debug, Serialize)]
pub enum Expr {
    Error,
    Var(ExprVar),
    Num(i64),
    Bool(bool),
    Lam(Vec<(ExprVar, Option<Type>)>, Box<Expr>),
    App(Box<Expr>, Vec<Expr>),
    BinOp(Box<Expr>, OpCode, Box<Expr>),
    FunInst(ExprVar, Vec<Type>), // Instantiate function at monomorphic type.
    Let(ExprVar, Option<Type>, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Record(Vec<(ExprVar, Expr)>),
    Proj(Box<Expr>, ExprVar),
    Variant(ExprCon, Box<Expr>),
    Match(Box<Expr>, Vec<Branch>),
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

impl Default for Type {
    fn default() -> Self {
        Self::Error
    }
}

impl Default for Expr {
    fn default() -> Self {
        Self::Error
    }
}

impl TypeVar {
    pub fn new(x: &str) -> Self {
        Self(x.to_owned())
    }
}

impl fmt::Debug for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("t#{}", self.0))
    }
}

impl ExprVar {
    pub fn new(x: &str) -> Self {
        Self(x.to_owned())
    }
}

impl fmt::Debug for ExprVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("e#{}", self.0))
    }
}

impl ExprCon {
    pub fn new(x: &str) -> Self {
        Self(x.to_owned())
    }
}

impl fmt::Debug for ExprCon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("c#{}", self.0))
    }
}
