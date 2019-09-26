pub enum Expr {
    Num(i64),
    Op(Box<Expr>, Opcode, Box<Expr>),
}

pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
}
