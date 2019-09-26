use crate::syntax::*;

impl Expr {
    pub fn eval(&self) -> i64 {
        use Expr::*;
        match self {
            Num(n) => *n,
            Op(l, x, r) => x.eval(l.eval(), r.eval()),
        }
    }
}

impl Opcode {
    fn eval(&self, l: i64, r: i64) -> i64 {
        use Opcode::*;
        match self {
            Add => l + r,
            Sub => l - r,
            Mul => l * r,
            Div => l / r,
        }
    }
}
