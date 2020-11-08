mod debruijn;
mod iter;

#[derive(Clone, Debug)]
pub struct Module {
    pub decls: Vec<Decl>,
}

#[derive(Clone, Debug)]
pub enum Decl {
    Type(TypeDecl),
    Func(FuncDecl),
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
    pub name: TypeVar,
    pub body: Type,
}

#[derive(Clone, Debug)]
pub struct FuncDecl {
    pub name: ExprVar,
    pub type_params: Vec<TypeVar>,
    pub expr_params: Vec<(ExprVar, Type)>,
    pub return_type: Type,
    pub body: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeVar(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExprVar(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExprCon(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Var(TypeVar),
    Syn(TypeVar),
    Int,
    Bool,
    Fun(Vec<Type>, Box<Type>),
    App(TypeVar, Vec<Type>),
    Abs(Vec<TypeVar>, Box<Type>),
    Record(Vec<(ExprVar, Type)>),
    Variant(Vec<(ExprCon, Option<Type>)>),
}

#[derive(Clone, Debug)]
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
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub con: ExprCon,
    pub var: Option<ExprVar>,
    pub rhs: Expr,
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
