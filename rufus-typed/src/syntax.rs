use lalrpop_intern::InternedString;
use serde::{Serialize, Serializer};
use std::fmt;

mod debruijn;
mod iter;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Span<Pos = usize> {
    pub start: Pos,
    pub end: Pos,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Located<T, Pos = usize> {
    pub locatee: T,
    pub span: Span<Pos>,
}

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
    pub name: LTypeVar,
    pub params: Vec<LTypeVar>,
    pub body: LType,
}

#[derive(Clone, Debug, Serialize)]
pub struct FuncDecl {
    pub name: LExprVar,
    pub type_params: Vec<LTypeVar>,
    pub expr_params: Vec<(LExprVar, LType)>,
    pub return_type: LType,
    pub body: LExpr,
}

#[derive(Clone, Debug, Serialize)]
pub enum Type {
    Error,
    Var(LTypeVar),
    SynApp(LTypeVar, Vec<LType>),
    Int,
    Bool,
    Fun(Vec<LType>, Box<LType>),
    Record(Vec<(LExprVar, LType)>),
    Variant(Vec<(LExprCon, LType)>),
}

pub type LType = Located<Type>;

#[derive(Clone, Debug, Serialize)]
pub enum Expr {
    Error,
    Var(LExprVar),
    Num(i64),
    Bool(bool),
    Lam(Vec<(LExprVar, Option<LType>)>, Box<LExpr>),
    App(Box<LExpr>, Vec<LExpr>),
    BinOp(Box<LExpr>, OpCode, Box<LExpr>),
    FunInst(LExprVar, Vec<LType>), // Instantiate function at monomorphic type.
    Let(LExprVar, Option<LType>, Box<LExpr>, Box<LExpr>),
    If(Box<LExpr>, Box<LExpr>, Box<LExpr>),
    Record(Vec<(LExprVar, LExpr)>),
    Proj(Box<LExpr>, LExprVar),
    Variant(LExprCon, Box<LExpr>),
    Match(Box<LExpr>, Vec<LBranch>),
}

pub type LExpr = Located<Expr>;

#[derive(Clone, Debug, Serialize)]
pub struct Branch {
    pub con: LExprCon,
    pub var: Option<LExprVar>,
    pub rhs: LExpr,
}

pub type LBranch = Located<Branch>;

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

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct TypeVar(InternedString);

pub type LTypeVar = Located<TypeVar>;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ExprVar(InternedString);

pub type LExprVar = Located<ExprVar>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ExprCon(InternedString);

pub type LExprCon = Located<ExprCon>;

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
        Self(lalrpop_intern::intern(x))
    }
}

impl fmt::Debug for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("t#{}", self.0))
    }
}

impl Serialize for TypeVar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        lalrpop_intern::read(|interner| interner.data(self.0).serialize(serializer))
    }
}

impl ExprVar {
    pub fn new(x: &str) -> Self {
        Self(lalrpop_intern::intern(x))
    }
}

impl fmt::Debug for ExprVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("e#{}", self.0))
    }
}

impl Serialize for ExprVar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        lalrpop_intern::read(|interner| interner.data(self.0).serialize(serializer))
    }
}

impl ExprCon {
    pub fn new(x: &str) -> Self {
        Self(lalrpop_intern::intern(x))
    }
}

impl fmt::Debug for ExprCon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("c#{}", self.0))
    }
}

impl Serialize for ExprCon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        lalrpop_intern::read(|interner| interner.data(self.0).serialize(serializer))
    }
}

impl<T, Pos> Located<T, Pos> {
    pub fn new(locatee: T, span: Span<Pos>) -> Self {
        Self { locatee, span }
    }
}

impl<T> Located<T, usize> {
    pub fn gen(locatee: T) -> Self {
        Self::new(locatee, Span { start: 0, end: 0 })
    }
}

impl<T, Pos: Copy> Located<T, Pos> {
    pub fn as_ref(&self) -> Located<&T, Pos> {
        Located {
            locatee: &self.locatee,
            span: self.span,
        }
    }
}
