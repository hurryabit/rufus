use std::fmt;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Name(pub String);

#[derive(Clone)]
pub enum Expr {
    Num(i64),
    Var(Name),
    Op(Opcode, Box<Expr>, Box<Expr>),
    App(Name, Vec<Expr>),
    Let(Name, Box<Expr>, Box<Expr>),
    Lam(Vec<Name>, Box<Expr>),
}

#[derive(Clone)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
