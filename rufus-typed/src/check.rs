use crate::syntax;
pub use error::{Error, LError};
use std::collections;
use std::hash::Hash;
use std::rc::Rc;
use syntax::*;
use types::*;

mod error;
pub mod types;

type Arity = usize;

type Type = types::Type;

#[derive(Clone)]
pub struct KindEnv {
    builtin_types: Rc<collections::HashMap<TypeVar, Box<dyn Fn() -> syntax::Type>>>,
    types: Rc<collections::HashMap<TypeVar, TypeScheme>>,
    type_vars: im::HashSet<TypeVar>,
}

#[derive(Clone)]
pub struct TypeEnv {
    kind_env: KindEnv,
    funcs: Rc<collections::HashMap<ExprVar, TypeScheme>>,
    expr_vars: im::HashMap<ExprVar, RcType>,
}

impl Module {
    pub fn check(&mut self) -> Result<(), LError> {
        let mut builtin_types = collections::HashMap::new();
        builtin_types.insert(
            TypeVar::new("Int"),
            Box::new(|| syntax::Type::Int) as Box<dyn Fn() -> syntax::Type>,
        );
        builtin_types.insert(
            TypeVar::new("Bool"),
            Box::new(|| syntax::Type::Bool) as Box<dyn Fn() -> syntax::Type>,
        );

        if let Some((span, name)) = find_duplicate(self.type_decls().map(|decl| decl.name.as_ref()))
        {
            return Err(Located::new(
                Error::DuplicateTypeDecl {
                    var: *name.locatee,
                    original: span,
                },
                name.span,
            ));
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
            .map(|decl| Ok((decl.name.locatee, decl.check_signature(&kind_env)?)))
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
                    name.locatee,
                    TypeScheme {
                        params: params.iter().map(|param| param.locatee).collect(),
                        body: RcType::from_lsyntax(body),
                    },
                )
            })
            .collect()
    }
}

impl TypeDecl {
    pub fn check(&mut self, env: &KindEnv) -> Result<(), LError> {
        let Self {
            name: _,
            params,
            body,
        } = self;
        LTypeVar::check_unique(params.iter())?;
        let env = &mut env.clone();
        env.type_vars = params.iter().map(|param| param.locatee).collect();
        body.check(env)
    }
}

impl FuncDecl {
    pub fn check_signature(&mut self, env: &KindEnv) -> Result<TypeScheme, LError> {
        let Self {
            name: _,
            type_params,
            expr_params,
            return_type,
            body: _,
        } = self;
        LTypeVar::check_unique(type_params.iter())?;
        let env = &mut env.clone();
        env.type_vars = type_params.iter().map(|param| param.locatee).collect();
        for (_, typ) in expr_params.iter_mut() {
            typ.check(env)?;
        }
        return_type.check(env)?;
        Ok(TypeScheme {
            params: type_params.iter().map(|param| param.locatee).collect(),
            body: RcType::new(Type::Fun(
                expr_params
                    .iter()
                    .map(|(_, typ)| RcType::from_lsyntax(typ))
                    .collect(),
                RcType::from_lsyntax(return_type),
            )),
        })
    }

    pub fn check(&mut self, env: &TypeEnv) -> Result<(), LError> {
        let Self {
            name: _,
            type_params,
            expr_params,
            return_type,
            body,
        } = self;
        LExprVar::check_unique(expr_params.iter().map(|(var, _)| var))?;
        let env = &mut env.clone();
        env.kind_env.type_vars = type_params.iter().map(|param| param.locatee).collect();
        env.expr_vars = expr_params
            .iter()
            .map(|(var, typ)| (var.locatee, RcType::from_lsyntax(typ)))
            .collect();
        body.check(env, &RcType::from_lsyntax(return_type))?;
        Ok(())
    }
}

impl LType {
    fn check(&mut self, env: &KindEnv) -> Result<(), LError> {
        self.locatee.check(self.span, env)
    }
}

impl syntax::Type {
    fn check(&mut self, span: Span, env: &KindEnv) -> Result<(), LError> {
        match self {
            Self::Error => Ok(()),
            Self::Int => panic!("Int in Type.check"),
            Self::Bool => panic!("Bool in Type.check"),
            Self::Var(var) => {
                if env.type_vars.contains(var) {
                    Ok(())
                } else if let Some(scheme) = env.types.get(var) {
                    let arity = scheme.params.len();
                    if arity == 0 {
                        *self = Self::SynApp(Located::new(*var, span), vec![]);
                        Ok(())
                    } else {
                        Err(Located::new(Error::UnexpectedGeneric(*var, arity), span))
                    }
                } else if let Some(builtin) = env.builtin_types.get(var) {
                    *self = builtin();
                    Ok(())
                } else {
                    Err(Located::new(Error::UnknownTypeVar(*var), span))
                }
            }
            Self::SynApp(var, args) => {
                let num_args = args.len();
                assert!(num_args > 0);
                if env.type_vars.contains(&var.locatee) {
                    Err(Located::new(
                        Error::GenericTypeArityMismatch {
                            type_var: var.locatee,
                            expected: 0,
                            found: num_args,
                        },
                        span,
                    ))
                } else if let Some(scheme) = env.types.get(&var.locatee) {
                    let arity = scheme.params.len();
                    if arity == num_args {
                        for arg in args {
                            arg.check(env)?;
                        }
                        Ok(())
                    } else {
                        Err(Located::new(
                            Error::GenericTypeArityMismatch {
                                type_var: var.locatee,
                                expected: arity,
                                found: num_args,
                            },
                            span,
                        ))
                    }
                } else if env.builtin_types.contains_key(&var.locatee) {
                    Err(Located::new(
                        Error::GenericTypeArityMismatch {
                            type_var: var.locatee,
                            expected: 0,
                            found: num_args,
                        },
                        span,
                    ))
                } else {
                    Err(Located::new(Error::UnknownTypeVar(var.locatee), span))
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

impl LExpr {
    pub fn check(&mut self, env: &TypeEnv, expected: &RcType) -> Result<(), LError> {
        self.locatee.check(self.span, env, expected)
    }

    fn infer(&mut self, env: &TypeEnv) -> Result<RcType, LError> {
        self.locatee.infer(self.span, env)
    }
}

impl Expr {
    pub fn check(&mut self, span: Span, env: &TypeEnv, expected: &RcType) -> Result<(), LError> {
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
                                let found = RcType::from_lsyntax(type_ann);
                                if !found.equiv(expected, &env.kind_env.types) {
                                    return Err(Located::new(
                                        Error::TypeMismatch {
                                            found,
                                            expected: expected.clone(),
                                        },
                                        span,
                                    ));
                                }
                                env.expr_vars.insert(var.locatee, found);
                            } else {
                                *opt_type_ann = Some(Located::new(expected.to_syntax(), var.span));
                                env.expr_vars.insert(var.locatee, expected.clone());
                            }
                        }
                        body.check(env, result)
                    }
                    _ => Err(Located::new(
                        Error::BadLam(expected.clone(), params.len()),
                        span,
                    )),
                }
            }
            Self::Let(binder, opt_type_ann, bindee, body) => {
                let binder_typ = check_let_bindee(env, binder, opt_type_ann, bindee)?;
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
                    if let Some(arg_typ) = find_by_key(&cons, &con.locatee) {
                        arg.check(env, arg_typ)
                    } else {
                        Err(Located::new(
                            Error::BadVariantConstr(expected.clone(), con.locatee),
                            span,
                        ))
                    }
                }
                _ => Err(Located::new(
                    Error::UnexpectedVariantType(expected.clone(), con.locatee),
                    span,
                )),
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
                            Err(Located::new(Error::EmptyMatch, span))
                        }
                    }
                    _ => Err(Located::new(Error::BadMatch(scrut_typ), span)),
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
                let found = self.infer(span, env)?;
                if found.equiv(expected, &env.kind_env.types) {
                    Ok(())
                } else {
                    Err(Located::new(
                        Error::TypeMismatch {
                            found: found.clone(),
                            expected: expected.clone(),
                        },
                        span,
                    ))
                }
            }
        }
    }

    fn infer(&mut self, span: Span, env: &TypeEnv) -> Result<RcType, LError> {
        match self {
            Self::Error => Ok(RcType::new(Type::Error)),
            Self::Var(var) => {
                if let Some(found) = env.expr_vars.get(var) {
                    Ok(found.clone())
                } else if let Some(TypeScheme { params, body }) = env.funcs.get(var) {
                    let arity = params.len();
                    if arity == 0 {
                        *self = Self::FunInst(Located::new(*var, span), vec![]);
                        Ok(body.clone())
                    } else {
                        Err(Located::new(
                            Error::GenericFuncArityMismatch {
                                expr_var: *var,
                                expected: 0,
                                found: arity,
                            },
                            span,
                        ))
                    }
                } else {
                    Err(Located::new(Error::UnknownExprVar(*var), span))
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
                        let typ = RcType::from_lsyntax(opt_type_ann.as_ref().unwrap());
                        env.expr_vars.insert(var.locatee, typ.clone());
                        typ
                    })
                    .collect();
                let result = body.infer(env)?;
                Ok(RcType::new(Type::Fun(param_types, result)))
            }
            Self::App(func, args) => {
                let func_type = func.infer(env)?;
                let num_args = args.len();
                match &*func_type.weak_normalize_env(env) {
                    Type::Fun(params, result) if params.len() == num_args => {
                        for (arg, typ) in args.iter_mut().zip(params.iter()) {
                            arg.check(env, typ)?;
                        }
                        Ok(result.clone())
                    }
                    _ => {
                        let func = match func.locatee {
                            Expr::Var(var) => Some(var),
                            Expr::FunInst(func, _) => Some(func.locatee),
                            _ => None,
                        };
                        Err(Located::new(
                            Error::BadApp {
                                func,
                                func_type,
                                num_args,
                            },
                            span,
                        ))
                    }
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
                if env.expr_vars.contains_key(&var.locatee) {
                    Err(Located::new(
                        Error::GenericFuncArityMismatch {
                            expr_var: var.locatee,
                            expected: 0,
                            found: num_types,
                        },
                        span,
                    ))
                } else if let Some(scheme) = env.funcs.get(&var.locatee) {
                    let arity = scheme.params.len();
                    if arity == num_types {
                        let types = types.iter().map(RcType::from_lsyntax).collect();
                        Ok(scheme.instantiate(&types))
                    } else {
                        Err(Located::new(
                            Error::GenericFuncArityMismatch {
                                expr_var: var.locatee,
                                expected: arity,
                                found: num_types,
                            },
                            span,
                        ))
                    }
                } else {
                    Err(Located::new(Error::UnknownExprVar(var.locatee), span))
                }
            }
            Self::Let(binder, opt_type_ann, bindee, body) => {
                let binder_typ = check_let_bindee(env, binder, opt_type_ann, bindee)?;
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
                    .map(|(name, expr)| Ok((name.locatee, expr.infer(env)?)))
                    .collect::<Result<_, _>>()?;
                Ok(RcType::new(Type::Record(fields)))
            }
            Self::Proj(record, field) => {
                let record_type = record.infer(env)?;
                let field = field.locatee;
                match &*record_type.weak_normalize_env(env) {
                    Type::Record(fields) => {
                        if let Some(field_typ) = find_by_key(&fields, &field) {
                            Ok(field_typ.clone())
                        } else {
                            Err(Located::new(
                                Error::BadRecordProj { record_type, field },
                                span,
                            ))
                        }
                    }
                    _ => Err(Located::new(
                        Error::BadRecordProj { record_type, field },
                        span,
                    )),
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
                            Err(Located::new(Error::EmptyMatch, span))
                        }
                    }
                    _ => Err(Located::new(Error::BadMatch(scrut_typ), span)),
                }
            }
            Self::Lam(_, _) | Self::Variant(_, _) => Err(Located::new(Error::TypeAnnsNeeded, span)),
        }
    }
}

impl LBranch {
    fn infer(
        &mut self,
        env: &TypeEnv,
        scrut_type: &RcType,
        cons: &Vec<(ExprCon, RcType)>,
    ) -> Result<RcType, LError> {
        self.locatee.infer(self.span, env, scrut_type, cons)
    }

    fn check(
        &mut self,
        env: &TypeEnv,
        scrut_type: &RcType,
        cons: &Vec<(ExprCon, RcType)>,
        expected: &RcType,
    ) -> Result<(), LError> {
        self.locatee
            .check(self.span, env, scrut_type, cons, expected)
    }
}

impl Branch {
    fn infer(
        &mut self,
        span: Span,
        env: &TypeEnv,
        scrut_type: &RcType,
        cons: &Vec<(ExprCon, RcType)>,
    ) -> Result<RcType, LError> {
        if let Some(arg_type) = find_by_key(cons, &self.con.locatee) {
            if let Some(var) = &self.var {
                self.rhs.infer(&env.intro_expr_var(var, arg_type.clone()))
            } else {
                self.rhs.infer(env)
            }
        } else {
            Err(Located::new(
                Error::BadBranch(scrut_type.clone(), self.con.locatee),
                span,
            ))
        }
    }

    fn check(
        &mut self,
        span: Span,
        env: &TypeEnv,
        scrut_type: &RcType,
        cons: &Vec<(ExprCon, RcType)>,
        expected: &RcType,
    ) -> Result<(), LError> {
        if let Some(arg_type) = find_by_key(cons, &self.con.locatee) {
            if let Some(var) = &self.var {
                self.rhs
                    .check(&env.intro_expr_var(var, arg_type.clone()), expected)
            } else {
                self.rhs.check(env, expected)
            }
        } else {
            Err(Located::new(
                Error::BadBranch(scrut_type.clone(), self.con.locatee),
                span,
            ))
        }
    }
}

impl TypeEnv {
    fn intro_expr_var(&self, var: &LExprVar, typ: RcType) -> Self {
        let mut env = self.clone();
        env.expr_vars.insert(var.locatee, typ);
        env
    }
}

impl LTypeVar {
    fn check_unique<'a, I: Iterator<Item = &'a LTypeVar>>(iter: I) -> Result<(), LError> {
        if let Some((span, lvar)) = find_duplicate(iter.map(Located::as_ref)) {
            Err(Located::new(
                Error::DuplicateTypeVar {
                    var: *lvar.locatee,
                    original: span,
                },
                span,
            ))
        } else {
            Ok(())
        }
    }
}

impl LExprVar {
    fn check_unique<'a, I: Iterator<Item = &'a LExprVar>>(iter: I) -> Result<(), LError> {
        if let Some((span, lvar)) = find_duplicate(iter.map(Located::as_ref)) {
            Err(Located::new(
                Error::DuplicateExprVar {
                    var: *lvar.locatee,
                    original: span,
                },
                span,
            ))
        } else {
            Ok(())
        }
    }
}

fn check_lam_params(
    params: &mut Vec<(LExprVar, Option<syntax::LType>)>,
    env: &TypeEnv,
) -> Result<(), LError> {
    for (_, opt_typ) in params.iter_mut() {
        if let Some(typ) = opt_typ {
            typ.check(&env.kind_env)?;
        }
    }
    LExprVar::check_unique(params.iter().map(|(name, _)| name))
}

fn check_let_bindee(
    env: &TypeEnv,
    binder: &LExprVar,
    opt_type_ann: &mut Option<syntax::LType>,
    bindee: &mut LExpr,
) -> Result<RcType, LError> {
    if let Some(type_ann) = opt_type_ann {
        let typ = RcType::from_lsyntax(type_ann);
        bindee.check(env, &typ)?;
        Ok(typ)
    } else {
        let typ = bindee.infer(env)?;
        *opt_type_ann = Some(Located::new(typ.to_syntax(), binder.span));
        Ok(typ)
    }
}

fn find_duplicate<'a, T: Eq + Hash, I: Iterator<Item = Located<T>>>(
    iter: I,
) -> Option<(Span, Located<T>)> {
    let mut seen = std::collections::HashMap::new();
    for lvar in iter {
        if let Some(span) = seen.get(&lvar.locatee) {
            return Some((*span, lvar));
        } else {
            seen.insert(lvar.locatee, lvar.span);
        }
    }
    None
}

fn find_by_key<'a, K: Eq, V>(vec: &'a Vec<(K, V)>, key: &K) -> Option<&'a V> {
    vec.iter()
        .find_map(|(k, v)| if k == key { Some(v) } else { None })
}
