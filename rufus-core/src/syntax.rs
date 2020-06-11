use std::fmt;

mod debruijn;
mod iter;

use debruijn::Indexer;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Name(pub String);

#[derive(Clone, Debug)]
pub enum Expr {
    Var(Name, Option<usize>),
    Num(i64),
    Bool(bool),
    PrimOp(OpCode),
    App(Box<Expr>, Vec<Expr>),
    Lam(Vec<Name>, Box<Expr>),
    Let(Name, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Record(Vec<Name>, Vec<Expr>),
    Proj(Box<Expr>, Name),
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

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Expr {
    pub fn index(mut self) -> Result<Self, String> {
        self.index_aux(&mut Indexer::new())?;
        Ok(self)
    }

    fn index_aux(&mut self, indexer: &mut Indexer) -> Result<(), String> {
        use Expr::*;
        match self {
            Var(x, i @ None) => {
                if let Some(j) = indexer.get(&x.0) {
                    *i = Some(j);
                } else {
                    return Err(format!("unbound variable: {}", x.0));
                }
            }
            Var(_, Some(_)) => panic!("indexer running on indexed expression"),
            Lam(xs, e) => {
                // TODO(MH): Make this more efficient by using iterators.
                indexer.intro_many(
                    &xs.iter().map(|x| x.0.as_ref()).collect::<Vec<&str>>(),
                    |indexer| e.index_aux(indexer),
                )?;
            }
            Let(x, e1, e2) => {
                e1.index_aux(indexer)?;
                indexer.intro(&x.0, |indexer| e2.index_aux(indexer))?;
            }
            _ => {
                for e in self.children_mut() {
                    e.index_aux(indexer)?;
                }
            }
        }
        Ok(())
    }
}

impl OpCode {
    pub fn arity(self) -> usize {
        use OpCode::*;
        match self {
            Add | Sub | Mul | Div | Equals | NotEq | Less | LessEq | Greater | GreaterEq => 2,
        }
    }
}
