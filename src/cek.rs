use std::rc::Rc;

use crate::syntax::*;

#[derive(Debug)]
pub enum Value<'a> {
    Num(i64),
    Lam(usize, &'a Expr, Env<'a>),
}

#[derive(Clone, Debug)]
pub struct Env<'a> {
    stack: Vec<Rc<Value<'a>>>,
}

#[derive(Debug)]
enum Prim<'a> {
    Builtin(&'a Opcode),
    Lam(&'a Expr, Env<'a>),
}

#[derive(Debug)]
struct PAP<'a> {
    prim: Prim<'a>,
    args: Vec<Rc<Value<'a>>>,
    missing: usize,
}

#[derive(Debug)]
enum Ctrl<'a> {
    Evaluating,
    Expr(&'a Expr),
    PAP(PAP<'a>),
    Value(Rc<Value<'a>>),
}

#[derive(Debug)]
enum Kont<'a> {
    Dump(Env<'a>),
    Pop(usize),
    Arg(&'a Expr),
    PAP(PAP<'a>),
    Exec,
    Let(&'a Name, &'a Expr),
}

#[derive(Debug)]
pub struct State<'a> {
    ctrl: Ctrl<'a>,
    env: Env<'a>,
    kont: Vec<Kont<'a>>,
}

impl<'a> Value<'a> {
    pub fn as_i64(&self) -> i64 {
        if let Value::Num(n) = self {
            *n
        } else {
            panic!("expected i64, found {:?}", self)
        }
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

    pub fn push_many(&mut self, args: &[Rc<Value<'a>>]) {
        self.stack.extend_from_slice(args);
    }

    pub fn pop_many(&mut self, count: usize) {
        let new_len = self.stack.len() - count;
        self.stack.truncate(new_len);
    }
}

impl<'a> Ctrl<'a> {
    fn from_value(v: Value<'a>) -> Self {
        Ctrl::Value(Rc::new(v))
    }

    fn from_prim(prim: Prim<'a>, arity: usize) -> Self {
        Ctrl::PAP(PAP {
            prim,
            args: Vec::new(),
            missing: arity,
        })
    }
}

impl<'a> State<'a> {
    pub fn init(expr: &'a Expr) -> Self {
        State {
            ctrl: Ctrl::Expr(expr),
            env: Env::new(),
            kont: Vec::new(),
        }
    }

    /// Step when the control contains an expression.
    fn step_expr(&mut self, ctrl_expr: &'a Expr) -> Ctrl<'a> {
        match ctrl_expr {
            Expr::Num(n) => Ctrl::from_value(Value::Num(*n)),

            Expr::Var(_, None) => panic!("unindexed variable"),
            Expr::Var(_, Some(index)) => {
                let v = self.env.get(*index);
                Ctrl::Value(Rc::clone(v))
            }

            Expr::Op(op, x, y) => {
                self.kont.push(Kont::Arg(y));
                self.kont.push(Kont::Arg(x));
                Ctrl::from_prim(Prim::Builtin(op), 2)
            }

            Expr::App(fun, args) => {
                self.kont.extend(args.iter().rev().map(Kont::Arg));
                self.kont.push(Kont::Exec);
                Ctrl::Expr(fun)
            }

            Expr::Let(binder, bound, body) => {
                self.kont.push(Kont::Let(binder, body));
                Ctrl::Expr(bound)
            }

            Expr::Lam(params, body) => {
                Ctrl::from_value(Value::Lam(params.len(), body, self.env.clone()))
            }
        }
    }

    fn step_saturated_prim(&mut self, prim: &Prim<'a>, args: &[Rc<Value<'a>>]) -> Ctrl<'a> {
        match prim {
            Prim::Builtin(op) => {
                assert_eq!(args.len(), 2);
                let x = args[0].as_i64();
                let y = args[1].as_i64();
                Ctrl::from_value(Value::Num(op.eval(x, y).expect("failing prim op")))
            }
            Prim::Lam(body, env) => {
                let mut new_env = env.clone();
                new_env.push_many(args);
                let old_env = std::mem::replace(&mut self.env, new_env);
                self.kont.push(Kont::Dump(old_env));
                Ctrl::Expr(body)
            }
        }
    }

    /// Step when the control contains a (proper) value.
    fn step_value(&mut self, value: Rc<Value<'a>>) -> Ctrl<'a> {
        let kont = self.kont.pop().expect("Step on final state");
        match kont {
            Kont::Dump(env) => {
                self.env = env;
                Ctrl::Value(value)
            }
            Kont::Pop(count) => {
                self.env.pop_many(count);
                Ctrl::Value(value)
            }
            Kont::Arg(_) => panic!("applying value"),
            Kont::PAP(mut pap) => {
                assert!(pap.missing > 0);
                pap.args.push(value);
                pap.missing -= 1;
                Ctrl::PAP(pap)
            }
            Kont::Exec => match &*value {
                Value::Lam(arity, ref body, ref env) => {
                    Ctrl::from_prim(Prim::Lam(body, env.clone()), *arity)
                }
                _ => panic!("executing non lambda"),
            },
            Kont::Let(_name, body) => {
                self.kont.push(Kont::Pop(1));
                self.env.push(value);
                Ctrl::Expr(body)
            }
        }
    }

    pub fn step(&mut self) {
        let old_ctrl = std::mem::replace(&mut self.ctrl, Ctrl::Evaluating);

        let new_ctrl = match old_ctrl {
            Ctrl::Evaluating => panic!("Control was not updated after last step"),

            Ctrl::Expr(expr) => self.step_expr(expr),

            Ctrl::Value(value) => self.step_value(value),
            Ctrl::PAP(pap) => {
                if pap.missing == 0 {
                    self.step_saturated_prim(&pap.prim, &pap.args)
                } else if let Some(Kont::Arg(arg)) = self.kont.pop() {
                    self.kont.push(Kont::PAP(pap));
                    Ctrl::Expr(arg)
                } else {
                    panic!("not enough args for PAP")
                }
            }
        };

        self.ctrl = new_ctrl
    }

    fn is_final(&self) -> bool {
        match &self.ctrl {
            Ctrl::Value(_) => self.kont.is_empty(),
            _ => false,
        }
    }

    pub fn run(mut self) -> Rc<Value<'a>> {
        while !self.is_final() {
            self.step();
        }
        match self.ctrl {
            Ctrl::Value(v) => v,
            _ => panic!("impossible"),
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

impl Opcode {
    pub fn eval(&self, x: i64, y: i64) -> Result<i64, String> {
        use Opcode::*;
        Ok(match self {
            Add => x + y,
            Sub => x - y,
            Mul => x * y,
            Div => x / y,
        })
    }
}
