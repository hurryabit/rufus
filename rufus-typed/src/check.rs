use crate::syntax;
use std::collections;
use std::hash::Hash;
use std::rc::Rc;
use syntax::{Branch, Expr, ExprCon, ExprVar, FuncDecl, Module, OpCode, TypeDecl, TypeVar};
use types::*;

pub mod types;

type Arity = usize;

#[derive(Clone)]
pub struct KindEnv {
    builtin_types: Rc<collections::HashMap<TypeVar, syntax::Type>>,
    types: Rc<collections::HashMap<TypeVar, TypeScheme>>,
    type_vars: im::HashSet<TypeVar>,
}

#[derive(Clone)]
pub struct TypeEnv {
    kind_env: KindEnv,
    funcs: Rc<collections::HashMap<ExprVar, TypeScheme>>,
    expr_vars: im::HashMap<ExprVar, RcType>,
}

#[derive(Debug)]
pub enum Error {
    UnknownTypeVar(TypeVar),
    UnknownExprVar(ExprVar),
    UnknownField(ExprVar),
    KindMismatch {
        type_var: TypeVar,
        expected: Arity,
        found: Arity,
    },
    SchemeMismatch {
        expr_var: ExprVar,
        expected: Arity,
        found: Arity,
    },
    TypeMismatch {
        expr: Expr,
        expected: RcType,
        found: RcType,
    },
    DuplicateTypeVar(TypeVar),
    DuplicateTypeDecl(TypeVar),
    DuplicateExprVar(ExprVar),
    BadRecordProj(RcType, ExprVar),
    BadApp(RcType, Arity),
    BadLam(RcType, Arity),
    BadVariant(RcType, ExprCon),
    BadMatch(RcType),
    BadBranch(RcType, ExprCon),
    EmptyMatch,
    TypeAnnsNeeded(Expr),
    NotImplemented(&'static str),
}

impl Module {
    pub fn check(&mut self) -> Result<(), Error> {
        let mut builtin_types = collections::HashMap::new();
        builtin_types.insert(TypeVar::new("Int"), syntax::Type::Int);
        builtin_types.insert(TypeVar::new("Bool"), syntax::Type::Bool);

        if let Some(name) = find_duplicate(self.type_decls().map(|decl| &decl.name)) {
            return Err(Error::DuplicateTypeDecl(*name));
        }

        let types = self.types();
        let type_vars = im::HashSet::new();
        let mut kind_env = KindEnv {
            builtin_types: Rc::new(builtin_types),
            types: Rc::new(types),
            type_vars,
        };
        for type_decl in self.type_decls_mut() {
            type_decl.check(&kind_env)?;
        }
        kind_env.types = Rc::new(self.types());
        let funcs = self
            .func_decls_mut()
            .map(|decl| Ok((decl.name, decl.check_signature(&kind_env)?)))
            .collect::<Result<_, _>>()?;
        let expr_vars = im::HashMap::new();
        let type_env = TypeEnv {
            kind_env,
            funcs: Rc::new(funcs),
            expr_vars,
        };
        for decl in self.func_decls_mut() {
            decl.check(&type_env)?;
        }
        Ok(())
    }

    fn types(&self) -> collections::HashMap<TypeVar, TypeScheme> {
        self.type_decls()
            .map(|TypeDecl { name, params, body }| {
                (
                    *name,
                    TypeScheme {
                        params: params.clone(),
                        body: RcType::from_syntax(body),
                    },
                )
            })
            .collect()
    }
}

impl TypeDecl {
    pub fn check(&mut self, env: &KindEnv) -> Result<(), Error> {
        let Self {
            name: _,
            params,
            body,
        } = self;
        TypeVar::check_unique(params.iter())?;
        let env = &mut env.clone();
        env.type_vars = params.iter().cloned().collect();
        body.check(env)
    }
}

impl FuncDecl {
    pub fn check_signature(&mut self, env: &KindEnv) -> Result<TypeScheme, Error> {
        let Self {
            name: _,
            type_params,
            expr_params,
            return_type,
            body: _,
        } = self;
        TypeVar::check_unique(type_params.iter())?;
        let env = &mut env.clone();
        env.type_vars = type_params.iter().cloned().collect();
        for (_, typ) in expr_params.iter_mut() {
            typ.check(env)?;
        }
        return_type.check(env)?;
        Ok(TypeScheme {
            params: type_params.clone(),
            body: RcType::new(Type::Fun(
                expr_params
                    .iter()
                    .map(|(_, typ)| RcType::from_syntax(typ))
                    .collect(),
                RcType::from_syntax(return_type),
            )),
        })
    }

    pub fn check(&mut self, env: &TypeEnv) -> Result<(), Error> {
        let Self {
            name: _,
            type_params,
            expr_params,
            return_type,
            body,
        } = self;
        ExprVar::check_unique(expr_params.iter().map(|(var, _)| var))?;
        let env = &mut env.clone();
        env.kind_env.type_vars = type_params.iter().copied().collect();
        env.expr_vars = expr_params
            .iter()
            .map(|(var, typ)| (*var, RcType::from_syntax(typ)))
            .collect();
        body.check(env, &RcType::from_syntax(return_type))?;
        Ok(())
    }
}

impl syntax::Type {
    fn check(&mut self, env: &KindEnv) -> Result<(), Error> {
        match self {
            Self::Error => Ok(()),
            Self::Int | Self::Bool => panic!("{:?} in Type.check", self),
            Self::Var(var) => {
                if env.type_vars.contains(var) {
                    Ok(())
                } else if let Some(scheme) = env.types.get(var) {
                    let arity = scheme.params.len();
                    if arity == 0 {
                        *self = Self::SynApp(*var, vec![]);
                        Ok(())
                    } else {
                        Err(Error::KindMismatch {
                            type_var: *var,
                            expected: 0,
                            found: arity,
                        })
                    }
                } else if let Some(builtin) = env.builtin_types.get(var) {
                    *self = builtin.clone();
                    Ok(())
                } else {
                    Err(Error::UnknownTypeVar(*var))
                }
            }
            Self::SynApp(var, args) => {
                let num_args = args.len();
                assert!(num_args > 0);
                if env.type_vars.contains(var) {
                    Err(Error::KindMismatch {
                        type_var: *var,
                        expected: num_args,
                        found: 0,
                    })
                } else if let Some(scheme) = env.types.get(var) {
                    let arity = scheme.params.len();
                    if arity == num_args {
                        for arg in args {
                            arg.check(env)?;
                        }
                        Ok(())
                    } else {
                        Err(Error::KindMismatch {
                            type_var: *var,
                            expected: num_args,
                            found: arity,
                        })
                    }
                } else if env.builtin_types.contains_key(var) {
                    Err(Error::KindMismatch {
                        type_var: *var,
                        expected: num_args,
                        found: 0,
                    })
                } else {
                    Err(Error::UnknownTypeVar(*var))
                }
            }
            Self::Fun(_, _) | Self::Record(_) | Self::Variant(_) => {
                for child in self.children_mut() {
                    child.check(env)?;
                }
                Ok(())
            }
        }
    }
}

impl RcType {
    pub fn weak_normalize_env(&self, env: &TypeEnv) -> Self {
        self.weak_normalize(&env.kind_env.types)
    }
}

impl Expr {
    pub fn check(&mut self, env: &TypeEnv, expected: &RcType) -> Result<(), Error> {
        match self {
            Self::Lam(params, body) if params.iter().any(|(_, opt_typ)| opt_typ.is_none()) => {
                check_lam_params(params, env)?;
                match &*expected.weak_normalize_env(env) {
                    Type::Fun(param_types, result) if params.len() == param_types.len() => {
                        let env = &mut env.clone();
                        // TODO(MH): Replace `x` with a pattern once
                        // https://github.com/rust-lang/rust/issues/68354
                        // has been stabilized.
                        for mut x in params.iter_mut().zip(param_types.iter()) {
                            let (var, opt_type_ann) = &mut x.0;
                            let expected = x.1;
                            if let Some(type_ann) = opt_type_ann {
                                let found = RcType::from_syntax(type_ann);
                                if !found.equiv(expected, &env.kind_env.types) {
                                    return Err(Error::TypeMismatch {
                                        expr: Expr::Var(*var),
                                        found,
                                        expected: expected.clone(),
                                    });
                                }
                                env.expr_vars.insert(*var, found);
                            } else {
                                *opt_type_ann = Some(expected.to_syntax());
                                env.expr_vars.insert(*var, expected.clone());
                            }
                        }
                        body.check(env, result)
                    }
                    _ => Err(Error::BadLam(expected.clone(), params.len())),
                }
            }
            Self::Let(binder, opt_type_ann, bindee, body) => {
                let binder_typ = check_let_bindee(env, opt_type_ann, bindee)?;
                body.check(&env.intro_expr_var(binder, binder_typ), expected)
            }
            Self::If(cond, then, elze) => {
                cond.check(env, &RcType::new(Type::Bool))?;
                then.check(env, &expected)?;
                elze.check(env, &expected)?;
                Ok(())
            }
            Self::Variant(con, arg) => match &*expected.weak_normalize_env(env) {
                Type::Variant(cons) => {
                    if let Some(arg_typ) = find_by_key(&cons, con) {
                        arg.check(env, arg_typ)
                    } else {
                        Err(Error::BadVariant(expected.clone(), *con))
                    }
                }
                _ => Err(Error::BadVariant(expected.clone(), *con)),
            },
            Self::Match(scrut, branches) => {
                let scrut_typ = scrut.infer(env)?;
                match &*scrut_typ.weak_normalize_env(env) {
                    Type::Variant(cons) => {
                        if !branches.is_empty() {
                            for branch in branches {
                                branch.check(env, &scrut_typ, cons, expected)?;
                            }
                            Ok(())
                        } else {
                            Err(Error::EmptyMatch)
                        }
                    }
                    _ => Err(Error::BadMatch(scrut_typ)),
                }
            }
            Self::Error
            | Self::Var(_)
            | Self::Num(_)
            | Self::Bool(_)
            | Self::Lam(_, _)
            | Self::App(_, _)
            | Self::BinOp(_, _, _)
            | Self::FunInst(_, _)
            | Self::Record(_)
            | Self::Proj(_, _) => {
                let found = self.infer(env)?;
                if found.equiv(expected, &env.kind_env.types) {
                    Ok(())
                } else {
                    Err(Error::TypeMismatch {
                        expr: self.clone(),
                        found: found.clone(),
                        expected: expected.clone(),
                    })
                }
            }
        }
    }

    fn infer(&mut self, env: &TypeEnv) -> Result<RcType, Error> {
        match self {
            Self::Error => Ok(RcType::new(Type::Error)),
            Self::Var(var) => {
                if let Some(found) = env.expr_vars.get(var) {
                    Ok(found.clone())
                } else if let Some(TypeScheme { params, body }) = env.funcs.get(var) {
                    let arity = params.len();
                    if arity == 0 {
                        *self = Self::FunInst(*var, vec![]);
                        Ok(body.clone())
                    } else {
                        Err(Error::SchemeMismatch {
                            expr_var: *var,
                            expected: 0,
                            found: arity,
                        })
                    }
                } else {
                    Err(Error::UnknownExprVar(*var))
                }
            }
            Self::Num(_) => Ok(RcType::new(Type::Int)),
            Self::Bool(_) => Ok(RcType::new(Type::Bool)),
            Self::Lam(params, body) if params.iter().all(|(_, opt_typ)| opt_typ.is_some()) => {
                check_lam_params(params, env)?;
                let env = &mut env.clone();
                let param_types = params
                    .iter()
                    .map(|(var, opt_type_ann)| {
                        let typ = RcType::from_syntax(opt_type_ann.as_ref().unwrap());
                        env.expr_vars.insert(*var, typ.clone());
                        typ
                    })
                    .collect();
                let result = body.infer(env)?;
                Ok(RcType::new(Type::Fun(param_types, result)))
            }
            Self::App(fun, args) => {
                let fun_type = fun.infer(env)?;
                match &*fun_type.weak_normalize_env(env) {
                    Type::Fun(params, result) if params.len() == args.len() => {
                        for (arg, typ) in args.iter_mut().zip(params.iter()) {
                            arg.check(env, typ)?;
                        }
                        Ok(result.clone())
                    }
                    _ => Err(Error::BadApp(fun_type, args.len())),
                }
            }
            Self::BinOp(lhs, op, rhs) => match op {
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                    let int = RcType::new(Type::Int);
                    lhs.check(env, &int)?;
                    rhs.check(env, &int)?;
                    Ok(int)
                }
                OpCode::Equals
                | OpCode::NotEq
                | OpCode::Less
                | OpCode::LessEq
                | OpCode::Greater
                | OpCode::GreaterEq => {
                    let typ = lhs.infer(env)?;
                    rhs.check(env, &typ)?;
                    Ok(RcType::new(Type::Bool))
                }
            },
            Self::FunInst(var, types) => {
                let num_types = types.len();
                assert!(num_types > 0);
                for typ in types.iter_mut() {
                    typ.check(&env.kind_env)?;
                }
                if env.expr_vars.contains_key(var) {
                    Err(Error::SchemeMismatch {
                        expr_var: *var,
                        expected: num_types,
                        found: 0,
                    })
                } else if let Some(scheme) = env.funcs.get(var) {
                    let arity = scheme.params.len();
                    if arity == num_types {
                        let types = types.iter().map(RcType::from_syntax).collect();
                        Ok(scheme.instantiate(&types))
                    } else {
                        Err(Error::SchemeMismatch {
                            expr_var: *var,
                            expected: num_types,
                            found: arity,
                        })
                    }
                } else {
                    Err(Error::UnknownExprVar(*var))
                }
            }
            Self::Let(binder, opt_type_ann, bindee, body) => {
                let binder_typ = check_let_bindee(env, opt_type_ann, bindee)?;
                body.infer(&env.intro_expr_var(binder, binder_typ))
            }
            Self::If(cond, then, elze) => {
                cond.check(env, &RcType::new(Type::Bool))?;
                let typ = then.infer(env)?;
                elze.check(env, &typ)?;
                Ok(typ)
            }
            Self::Record(fields) => {
                let fields = fields
                    .iter_mut()
                    .map(|(name, expr)| Ok((*name, expr.infer(env)?)))
                    .collect::<Result<_, _>>()?;
                Ok(RcType::new(Type::Record(fields)))
            }
            Self::Proj(record, field) => {
                let record_typ = record.infer(env)?;
                match &*record_typ.weak_normalize_env(env) {
                    Type::Record(fields) => {
                        if let Some(field_typ) = find_by_key(&fields, field) {
                            Ok(field_typ.clone())
                        } else {
                            Err(Error::BadRecordProj(record_typ, *field))
                        }
                    }
                    _ => Err(Error::BadRecordProj(record_typ, *field)),
                }
            }
            Self::Match(scrut, branches) => {
                let scrut_typ = scrut.infer(env)?;
                match &*scrut_typ.weak_normalize_env(env) {
                    Type::Variant(cons) => {
                        if let Some((first, rest)) = branches.split_first_mut() {
                            let rhs_typ = first.infer(env, &scrut_typ, cons)?;
                            for branch in rest {
                                branch.check(env, &scrut_typ, cons, &rhs_typ)?;
                            }
                            Ok(rhs_typ)
                        } else {
                            Err(Error::EmptyMatch)
                        }
                    }
                    _ => Err(Error::BadMatch(scrut_typ)),
                }
            }
            Self::Lam(_, _) | Self::Variant(_, _) => Err(Error::TypeAnnsNeeded(self.clone())),
        }
    }
}

impl Branch {
    fn infer(
        &mut self,
        env: &TypeEnv,
        scrut_type: &RcType,
        cons: &Vec<(ExprCon, RcType)>,
    ) -> Result<RcType, Error> {
        if let Some(arg_type) = find_by_key(cons, &self.con) {
            if let Some(var) = &self.var {
                self.rhs.infer(&env.intro_expr_var(var, arg_type.clone()))
            } else {
                self.rhs.infer(env)
            }
        } else {
            Err(Error::BadBranch(scrut_type.clone(), self.con))
        }
    }

    fn check(
        &mut self,
        env: &TypeEnv,
        scrut_type: &RcType,
        cons: &Vec<(ExprCon, RcType)>,
        expected: &RcType,
    ) -> Result<(), Error> {
        if let Some(arg_type) = find_by_key(cons, &self.con) {
            if let Some(var) = &self.var {
                self.rhs
                    .check(&env.intro_expr_var(var, arg_type.clone()), expected)
            } else {
                self.rhs.check(env, expected)
            }
        } else {
            Err(Error::BadBranch(scrut_type.clone(), self.con))
        }
    }
}

impl TypeEnv {
    fn intro_expr_var(&self, var: &ExprVar, typ: RcType) -> Self {
        let mut env = self.clone();
        env.expr_vars.insert(*var, typ);
        env
    }
}

impl TypeVar {
    fn check_unique<'a, I: Iterator<Item = &'a TypeVar>>(iter: I) -> Result<(), Error> {
        if let Some(dup) = find_duplicate(iter) {
            Err(Error::DuplicateTypeVar(*dup))
        } else {
            Ok(())
        }
    }
}

impl ExprVar {
    fn check_unique<'a, I: Iterator<Item = &'a ExprVar>>(iter: I) -> Result<(), Error> {
        if let Some(dup) = find_duplicate(iter) {
            Err(Error::DuplicateExprVar(*dup))
        } else {
            Ok(())
        }
    }
}

fn check_lam_params(
    params: &mut Vec<(ExprVar, Option<syntax::Type>)>,
    env: &TypeEnv,
) -> Result<(), Error> {
    for (_, opt_typ) in params.iter_mut() {
        if let Some(typ) = opt_typ {
            typ.check(&env.kind_env)?;
        }
    }
    ExprVar::check_unique(params.iter().map(|(name, _)| name))
}

fn check_let_bindee(
    env: &TypeEnv,
    opt_type_ann: &mut Option<syntax::Type>,
    bindee: &mut Expr,
) -> Result<RcType, Error> {
    if let Some(type_ann) = opt_type_ann {
        let typ = RcType::from_syntax(type_ann);
        bindee.check(env, &typ)?;
        Ok(typ)
    } else {
        let typ = bindee.infer(env)?;
        *opt_type_ann = Some(typ.to_syntax());
        Ok(typ)
    }
}

fn find_duplicate<T: Eq + Hash, I: Iterator<Item = T>>(iter: I) -> Option<T> {
    let mut seen = std::collections::HashSet::new();
    for item in iter {
        if seen.contains(&item) {
            return Some(item);
        } else {
            seen.insert(item);
        }
    }
    None
}

fn find_by_key<'a, K: Eq, V>(vec: &'a Vec<(K, V)>, key: &K) -> Option<&'a V> {
    vec.iter()
        .find_map(|(k, v)| if k == key { Some(v) } else { None })
}
