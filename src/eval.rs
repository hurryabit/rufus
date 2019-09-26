use crate::syntax::*;
use std::collections::HashMap;
use std::iter::Extend;
use std::rc::Rc;

pub enum Value {
    Num(i64),
    Lam(Vec<Name>, Box<Expr>, Env),
}

#[derive(Clone)]
pub struct Env(HashMap<Name, Rc<Value>>);

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
            Err("expected lambda, found somethingelse".to_string())
        }
    }
}

impl Env {
    pub fn new() -> Self {
        Env(HashMap::new())
    }
}

impl Expr {
    pub fn eval(&self, env: Env) -> Result<Rc<Value>, String> {
        use Expr::*;
        match self {
            Var(x) => match env.0.get(x) {
                None => Err(format!("unknown variable: {}", x)),
                Some(v) => Ok(v.clone()),
            },
            Num(n) => Ok(Rc::new(Value::Num(*n))),
            Op(o, x, y) => {
                let x = x.eval(env.clone())?.as_i64()?;
                let y = y.eval(env)?.as_i64()?;
                let z = o.eval(x, y)?;
                Ok(Rc::new(Value::Num(z)))
            }
            App(f, xs) => {
                let xs = xs
                    .iter()
                    .map(|x| x.eval(env.clone()))
                    .collect::<Result<Vec<Rc<Value>>, String>>()?;
                let lam = Var(f.clone()).eval(env)?;
                let (params, body, env) = lam.as_lam()?;
                let mut env = env.clone();
                if params.len() != xs.len() {
                    return Err(format!(
                        "{} applied to {} arguments, but expected {}",
                        f,
                        xs.len(),
                        params.len()
                    ));
                }
                env.0.extend(params.iter().cloned().zip(xs.into_iter()));
                body.eval(env)
            }
            Let(x, e1, e2) => {
                let v1 = e1.eval(env.clone())?;
                let mut env = env;
                env.0.insert(x.clone(), v1);
                e2.eval(env)
            }
            Lam(xs, e) => Ok(Rc::new(Value::Lam(xs.clone(), e.clone(), env))),
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
