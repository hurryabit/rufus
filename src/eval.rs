use crate::syntax::*;
use std::rc::Rc;

pub enum Value {
    Num(i64),
    Lam(Vec<Name>, Box<Expr>, Env),
}

#[derive(Clone)]
pub struct Env(Vec<Rc<Value>>);

impl Value {
    pub fn as_i64(&self) -> Result<i64, String> {
        if let Value::Num(n) = self {
            Ok(*n)
        } else {
            Err("expected i64, found something else".to_string())
        }
    }

    pub fn as_lam(&self) -> Result<(&Vec<Name>, &Expr, &Env), String> {
        if let Value::Lam(params, body, env) = self {
            Ok((params, body, env))
        } else {
            Err("expected lambda, found something else".to_string())
        }
    }
}

impl Env {
    pub fn new() -> Self {
        Env(Vec::new())
    }

    pub fn intro<T>(&mut self, x: Rc<Value>, f: impl FnOnce(&mut Self) -> T) -> T {
        self.intro_many(vec![x], f)
    }

    pub fn intro_many<T>(&mut self, mut xs: Vec<Rc<Value>>, f: impl FnOnce(&mut Self) -> T) -> T {
        let old_len = self.0.len();
        self.0.append(&mut xs);
        let res = f(self);
        self.0.truncate(old_len);
        res
    }

    pub fn get(&self, i: usize) -> Option<Rc<Value>> {
        self.0.get(self.0.len() - i).cloned()
    }
}

impl Expr {
    pub fn eval(&self) -> Result<Rc<Value>, String> {
        self.eval_aux(&mut Env::new())
    }

    fn eval_aux(&self, env: &mut Env) -> Result<Rc<Value>, String> {
        use Expr::*;
        match self {
            Var(x, i) => match i.and_then(|i| env.get(i)) {
                None => Err(format!("unknown variable: {}", x)),
                Some(v) => Ok(v),
            },
            Num(n) => Ok(Rc::new(Value::Num(*n))),
            Op(o, x, y) => {
                let x = x.eval_aux(env)?.as_i64()?;
                let y = y.eval_aux(env)?.as_i64()?;
                let z = o.eval(x, y)?;
                Ok(Rc::new(Value::Num(z)))
            }
            App(f, xs) => {
                let xs = xs
                    .iter()
                    .map(|x| x.eval_aux(env))
                    .collect::<Result<Vec<Rc<Value>>, String>>()?;
                let lam = f.eval_aux(env)?;
                let (params, body, env) = lam.as_lam()?;
                let mut env = env.clone();
                if params.len() != xs.len() {
                    return Err(format!(
                        "{:?} applied to {} arguments, but expected {}",
                        f,
                        xs.len(),
                        params.len()
                    ));
                }
                env.intro_many(xs, |env| body.eval_aux(env))
            }
            Let(_x, e1, e2) => {
                let v1 = e1.eval_aux(env)?;
                env.intro(v1, |env| e2.eval_aux(env))
            }
            Lam(xs, e) => Ok(Rc::new(Value::Lam(xs.clone(), e.clone(), env.clone()))),
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
