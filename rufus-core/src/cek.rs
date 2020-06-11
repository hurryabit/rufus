use std::collections::HashMap;
use std::rc::Rc;

use crate::syntax::*;

#[derive(Debug)]
pub enum Value<'a> {
    Num(i64),
    Bool(bool),
    PAP(PAP<'a>),
    Record(HashMap<&'a Name, Rc<Value<'a>>>),
    Fix(Rc<Value<'a>>),
}

#[derive(Clone, Debug)]
pub struct PAP<'a> {
    prim: Prim<'a>,
    arity: usize,
    args: Vec<Rc<Value<'a>>>,
}

#[derive(Clone, Debug)]
pub enum Prim<'a> {
    Builtin(OpCode),
    Lam(&'a Expr, Rc<Env<'a>>),
    Record(&'a Vec<Name>),
    Proj(&'a Name),
}

#[derive(Debug)]
enum Ctrl<'a> {
    Evaluating,
    Expr(&'a Expr),
    Value(Rc<Value<'a>>),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct Env<'a> {
    stack: Vec<Rc<Value<'a>>>,
}

#[derive(Debug)]
enum Kont<'a> {
    Dump(Env<'a>),
    Pop(usize),
    Arg(&'a Expr),
    ArgValue(Rc<Value<'a>>),
    App(Rc<Value<'a>>),
    Let(&'a Name, &'a Expr),
    If(&'a Expr, &'a Expr),
}

#[derive(Debug)]
pub struct Machine<'a> {
    ctrl: Ctrl<'a>,
    env: Env<'a>,
    kont: Vec<Kont<'a>>,
}

impl<'a> Value<'a> {
    pub fn as_i64(&self) -> Result<i64, String> {
        if let Value::Num(n) = self {
            Ok(*n)
        } else {
            Err(format!("expected i64, found {:?}", self))
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        if let Value::Bool(b) = self {
            Ok(*b)
        } else {
            Err(format!("expected bool, found {:?}", self))
        }
    }

    fn as_record(&self) -> Result<&HashMap<&'a Name, Rc<Value<'a>>>, String> {
        if let Value::Record(assigns) = self {
            Ok(assigns)
        } else {
            Err(format!("expected record, found {:?}", self))
        }
    }
}

impl<'a> Ctrl<'a> {
    fn from_value(v: Value<'a>) -> Self {
        Ctrl::Value(Rc::new(v))
    }

    fn from_prim(prim: Prim<'a>, arity: usize) -> Self {
        assert!(arity > 0);
        Self::from_value(Value::PAP(PAP {
            prim,
            arity,
            args: Vec::with_capacity(arity),
        }))
    }
}

impl<'a> Env<'a> {
    fn new() -> Self {
        Env { stack: Vec::new() }
    }

    fn get(&self, index: usize) -> &Rc<Value<'a>> {
        self.stack
            .get(self.stack.len() - index)
            .expect("bad de Bruijn index")
    }

    pub fn push(&mut self, value: Rc<Value<'a>>) {
        self.stack.push(value);
    }

    pub fn push_many(&mut self, args: Vec<Rc<Value<'a>>>) {
        self.stack.extend(args.into_iter());
    }

    pub fn pop_many(&mut self, count: usize) {
        let new_len = self.stack.len() - count;
        self.stack.truncate(new_len);
    }
}

impl<'a> Machine<'a> {
    pub fn new(expr: &'a Expr) -> Self {
        Machine {
            ctrl: Ctrl::Expr(expr),
            env: Env::new(),
            kont: Vec::new(),
        }
    }

    /// Step when the control contains an expression.
    fn step_expr(&mut self, ctrl_expr: &'a Expr) -> Ctrl<'a> {
        use Expr::*;

        match ctrl_expr {
            Var(_, None) => panic!("unindexed variable"),
            Var(_, Some(index)) => {
                let v = self.env.get(*index);
                Ctrl::Value(Rc::clone(v))
            }
            Num(n) => Ctrl::from_value(Value::Num(*n)),
            Bool(b) => Ctrl::from_value(Value::Bool(*b)),
            PrimOp(op) => Ctrl::from_prim(Prim::Builtin(*op), op.arity()),
            App(fun, args) => {
                self.kont.extend(args.iter().rev().map(Kont::Arg));
                Ctrl::Expr(fun)
            }
            Lam(params, body) => {
                Ctrl::from_prim(Prim::Lam(body, Rc::new(self.env.clone())), params.len())
            }
            Let(binder, bound, body) => {
                self.kont.push(Kont::Let(binder, body));
                Ctrl::Expr(bound)
            }
            If(cond, then, elze) => {
                self.kont.push(Kont::If(then, elze));
                Ctrl::Expr(cond)
            }
            Record(fields, exprs) => {
                if fields.len() > 0 {
                    self.kont.extend(exprs.iter().rev().map(Kont::Arg));
                    Ctrl::from_prim(Prim::Record(fields), fields.len())
                } else {
                    Ctrl::from_value(Value::Record(HashMap::new()))
                }
            }
            Proj(record, field) => {
                self.kont.push(Kont::Arg(record));
                Ctrl::from_prim(Prim::Proj(field), 1)
            }
        }
    }

    /// Enter a fully applied primitived.
    fn enter_prim(&mut self, prim: Prim<'a>, args: Vec<Rc<Value<'a>>>) -> Ctrl<'a> {
        use Prim::*;
        match prim {
            Builtin(op) => match op.eval(args) {
                Ok(v) => Ctrl::from_value(v),
                Err(e) => Ctrl::Error(e),
            },
            Lam(body, env) => {
                let mut new_env = match Rc::try_unwrap(env) {
                    Ok(env) => env,
                    Err(env) => env.as_ref().clone(),
                };
                new_env.push_many(args);
                let old_env = std::mem::replace(&mut self.env, new_env);
                self.kont.push(Kont::Dump(old_env));
                Ctrl::Expr(body)
            }
            Record(names) => Ctrl::from_value(Value::Record(
                names.into_iter().zip(args.into_iter()).collect(),
            )),
            Proj(field) => match args[0].as_record() {
                Ok(record) => {
                    if let Some(value) = record.get(field) {
                        Ctrl::Value(Rc::clone(value))
                    } else {
                        Ctrl::Error(format!("unknown field in record: {}", field))
                    }
                }
                Err(msg) => Ctrl::Error(msg),
            },
        }
    }

    /// Apply an argument to a PAP. If it is the last argument, enter the
    /// primitive.
    fn pap_apply_arg(&mut self, mut pap: PAP<'a>, arg: Rc<Value<'a>>) -> Ctrl<'a> {
        assert!(pap.args.len() < pap.arity);
        pap.args.push(arg);
        if (pap.args.len() == pap.arity) {
            self.enter_prim(pap.prim, pap.args)
        } else {
            Ctrl::from_value(Value::PAP(pap))
        }
    }

    fn fix_apply_arg(&mut self, fun: Rc<Value<'a>>, arg: Rc<Value<'a>>) -> Ctrl<'a> {
        self.kont.push(Kont::ArgValue(arg));
        self.kont.push(Kont::ArgValue(Rc::new(Value::Fix(Rc::clone(&fun)))));
        Ctrl::Value(fun)
    }

    /// Step when the control contains a value.
    fn step_value(&mut self, value: Rc<Value<'a>>, kont: Kont<'a>) -> Ctrl<'a> {
        use Kont::*;

        match kont {
            Dump(env) => {
                self.env = env;
                Ctrl::Value(value)
            }
            Pop(count) => {
                self.env.pop_many(count);
                Ctrl::Value(value)
            }
            Arg(arg) => {
                self.kont.push(App(value));
                Ctrl::Expr(arg)
            }
            ArgValue(arg) => {
                self.kont.push(App(value));
                Ctrl::Value(arg)
            }
            App(fun) => match Rc::try_unwrap(fun) {
                Ok(fun) => match fun {
                    Value::PAP(pap) => self.pap_apply_arg(pap, value),
                    Value::Fix(fun) => self.fix_apply_arg(fun, value),
                    _ => Ctrl::Error(format!("expected PAP, found {:?}", fun)),
                },
                Err(fun) => match &*fun {
                    Value::PAP(pap) => self.pap_apply_arg(pap.clone(), value),
                    Value::Fix(fun) => self.fix_apply_arg(Rc::clone(fun), value),
                    _ => Ctrl::Error(format!("expected PAP, found {:?}", fun)),
                },
            },
            Let(_name, body) => {
                self.kont.push(Kont::Pop(1));
                self.env.push(value);
                Ctrl::Expr(body)
            }
            If(then, elze) => match value.as_bool() {
                Ok(true) => Ctrl::Expr(then),
                Ok(false) => Ctrl::Expr(elze),
                Err(e) => Ctrl::Error(e),
            },
        }
    }

    /// Step through the machine until completion.
    pub fn run(mut self) -> Result<Rc<Value<'a>>, String> {
        use Ctrl::*;
        loop {
            let old_ctrl = std::mem::replace(&mut self.ctrl, Ctrl::Evaluating);
            let new_ctrl = match old_ctrl {
                Evaluating => panic!("control was not updated after last step"),
                Expr(expr) => self.step_expr(expr),
                Value(value) => match self.kont.pop() {
                    None => return Ok(value),
                    Some(kont) => self.step_value(value, kont),
                },
                Error(e) => return Err(e),
            };
            self.ctrl = new_ctrl
        }
    }

    //     #[allow(dead_code)]
    //     pub fn print_debug(&self) {
    //         println!("ctrl: {:?}", self.ctrl);
    //         println!("env:");
    //         for val in self.env.stack.iter().rev() {
    //             println!("# {:?}", val);
    //         }
    //         println!("kont:");
    //         for kont in self.kont.iter().rev() {
    //             println!("$ {:?}", kont);
    //         }
    //     }
}

impl OpCode {
    pub fn eval<'a>(self, args: Vec<Rc<Value<'a>>>) -> Result<Value<'a>, String> {
        use op_code::*;
        use std::ops::{Add, Div, Mul, Sub};
        use OpCode::*;

        match self {
            Add => eval_arith(i64::add, args),
            Sub => eval_arith(i64::sub, args),
            Mul => eval_arith(i64::mul, args),
            Div => eval_arith(i64::div, args),
            Equals => Ok(Value::Bool(eval_equals(args))),
            NotEq => Ok(Value::Bool(!eval_equals(args))),
            Less => eval_comp(i64::lt, args),
            LessEq => eval_comp(i64::le, args),
            Greater => eval_comp(i64::gt, args),
            GreaterEq => eval_comp(i64::ge, args),
            Fix => Ok(Value::Fix(Rc::clone(&args[0]))),
        }
    }
}

mod op_code {
    use super::*;

    pub fn eval_arith<'a, F: FnOnce(i64, i64) -> i64>(
        f: F,
        args: Vec<Rc<Value<'a>>>,
    ) -> Result<Value<'a>, String> {
        let x = args[0].as_i64()?;
        let y = args[1].as_i64()?;
        Ok(Value::Num(f(x, y)))
    }

    pub fn eval_equals<'a>(args: Vec<Rc<Value<'a>>>) -> bool {
        eval_equals2(&args[0], &args[1])
    }

    pub fn eval_equals2<'a>(x: &Rc<Value<'a>>, y: &Rc<Value<'a>>) -> bool {
        use Value::*;
        match (&**x, &**y) {
            (Num(x), Num(y)) => x == y,
            (Bool(x), Bool(y)) => x == y,
            (Record(x), Record(y)) => {
                use std::collections::HashSet;
                let x_keys = x.keys().collect::<HashSet<_>>();
                let y_keys = y.keys().collect::<HashSet<_>>();
                if x_keys != y_keys {
                    return false;
                }
                for key in x_keys {
                    if !(eval_equals2(x.get(key).unwrap(), y.get(key).unwrap())) {
                        return false;
                    }
                }
                true
            }
            (_, _) => false,
        }
    }

    pub fn eval_comp<'a, F: FnOnce(&i64, &i64) -> bool>(
        f: F,
        args: Vec<Rc<Value<'a>>>,
    ) -> Result<Value<'a>, String> {
        let x = args[0].as_i64()?;
        let y = args[1].as_i64()?;
        Ok(Value::Bool(f(&x, &y)))
    }
}
