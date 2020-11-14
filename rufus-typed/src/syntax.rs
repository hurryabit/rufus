use debug::DebugWriter;
use std::fmt;

mod debruijn;
mod debug;
#[macro_use]
mod ident;
mod iter;

#[derive(Clone, Copy, Debug)]
pub struct Span<Pos = usize> {
    pub start: Pos,
    pub end: Pos,
}

#[derive(Clone, Copy, Debug)]
pub struct Located<T, Pos = usize> {
    pub locatee: T,
    pub span: Span<Pos>,
}

pub struct Module {
    pub decls: Vec<Decl>,
}

pub enum Decl {
    Type(TypeDecl),
    Func(FuncDecl),
}

pub struct TypeDecl {
    pub name: LTypeVar,
    pub params: Vec<LTypeVar>,
    pub body: LType,
}

pub struct FuncDecl {
    pub name: LExprVar,
    pub type_params: Vec<LTypeVar>,
    pub expr_params: Vec<(LExprVar, LType)>,
    pub return_type: LType,
    pub body: LExpr,
}

pub enum Type {
    Error,
    Var(TypeVar),
    SynApp(LTypeVar, Vec<LType>),
    Int,
    Bool,
    Fun(Vec<LType>, Box<LType>),
    Record(Vec<(LExprVar, LType)>),
    Variant(Vec<(LExprCon, Option<LType>)>),
}

pub type LType = Located<Type>;

pub enum Expr {
    Error,
    Var(ExprVar),
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
    Variant(LExprCon, Option<Box<LExpr>>),
    Match(Box<LExpr>, Vec<Branch>),
}

pub type LExpr = Located<Expr>;

pub struct Branch {
    pub pattern: LPattern,
    pub body: LExpr,
}

pub struct Pattern {
    pub constr: ExprCon,
    pub binder: Option<LExprVar>,
}

pub type LPattern = Located<Pattern>;

#[derive(Clone, Copy)]
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

ident_type!(TypeVar);
pub type LTypeVar = Located<TypeVar>;

ident_type!(ExprVar);
pub type LExprVar = Located<ExprVar>;

ident_type!(ExprCon);
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

impl<Pos> Span<Pos> {
    pub fn map<Pos2, F: Fn(Pos) -> Pos2>(self, f: F) -> Span<Pos2> {
        Span {
            start: f(self.start),
            end: f(self.end),
        }
    }
}

impl<Pos: fmt::Display> fmt::Display for Span<Pos> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
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

impl<T, Pos> Located<T, Pos> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Located<U, Pos> {
        Located::new(f(self.locatee), self.span)
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

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugWriter::fmt(self, f)
    }
}

impl fmt::Debug for Decl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugWriter::fmt(self, f)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugWriter::fmt(self, f)
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugWriter::fmt(self, f)
    }
}
