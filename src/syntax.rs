pub enum Expr {
    Num(i64),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Fun(FunCode, Vec<Box<Expr>>),
}

pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum FunCode {
    Fac,
    Rem,
}
