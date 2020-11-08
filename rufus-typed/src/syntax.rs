mod debruijn;
mod iter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Var(Name),
    Synonym(Name),
    Int,
    Bool,
    Fun(Vec<Type>, Box<Type>),
    App(Name, Vec<Type>),
    Abs(Vec<Name>, Box<Type>),
    Record(Vec<(Name, Type)>),
    Variant(Vec<(Name, Option<Type>)>),
}

pub type Name = String;

#[derive(Clone, Debug)]
pub enum Expr {
    Var(Name),
    Num(i64),
    Bool(bool),
    Lam(Vec<Name>, Box<Expr>),
    FunApp(Box<Expr>, Vec<Expr>),
    OpApp(Box<Expr>, OpCode, Box<Expr>),
    TyLam(Vec<Name>, Box<Expr>),
    TyApp(Box<Expr>, Vec<Type>),
    Let(Name, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Record(Vec<Name>, Vec<Expr>),
    Proj(Box<Expr>, Name),
    Variant(Name, Option<Box<Expr>>),
    Match(Box<Expr>, Vec<Branch>),
}

#[derive(Clone, Debug)]
pub struct Branch {
    con: Name,
    var: Option<Name>,
    rhs: Expr,
}

#[derive(Clone, Copy, Debug)]
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
