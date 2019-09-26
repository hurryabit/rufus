use crate::syntax::*;

impl Expr {
    pub fn eval(&self) -> Result<i64, String> {
        use Expr::*;
        match self {
            Num(n) => Ok(*n),
            Op(x, o, y) => o.eval(x.eval()?, y.eval()?),
            Fun(f, xs) => {
                let ns = xs
                    .iter()
                    .map(|x| x.eval())
                    .collect::<Result<Vec<i64>, String>>()?;
                f.eval(&ns)
            }
        }
    }
}

impl Opcode {
    fn eval(&self, x: i64, y: i64) -> Result<i64, String> {
        use Opcode::*;
        Ok(match self {
            Add => x + y,
            Sub => x - y,
            Mul => x * y,
            Div => x / y,
        })
    }
}

impl FunCode {
    fn eval(&self, xs: &[i64]) -> Result<i64, String> {
        use FunCode::*;
        match (self, xs) {
            (Fac, [x]) => Ok(Self::fac(*x)),
            (Rem, [x, y]) => Ok(x % y),
            _ => Err(format!(
                "wrong number of arguments for {:?} (found {})",
                self,
                xs.len()
            )),
        }
    }

    fn fac(mut x: i64) -> i64 {
        let mut y = 1;
        while x > 0 {
            y *= x;
            x -= 1;
        }
        y
    }
}
