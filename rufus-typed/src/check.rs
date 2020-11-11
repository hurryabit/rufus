use crate::syntax::*;
use std::hash::Hash;

lazy_static::lazy_static! {
    static ref BUILTIN_TYPES: im::HashMap<TypeVar, Type> = {
        let mut res = im::HashMap::new();
        res.insert(TypeVar::new("Int"), Type::Int);
        res.insert(TypeVar::new("Bool"), Type::Bool);
        res
    };
}

type Arity = usize;

#[derive(Clone)]
pub struct KindEnv {
    type_syns: im::HashMap<TypeVar, Arity>,
    type_vars: im::HashSet<TypeVar>,
}

#[derive(Clone)]
pub struct TypeEnv {
    kind_env: KindEnv,
    funcs: im::HashMap<ExprVar, TypeScheme>,
    expr_vars: im::HashMap<ExprVar, Type>,
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
        expected: Type,
        found: Type,
    },
    DuplicateTypeVar(TypeVar),
    DuplicateTypeDecl(TypeVar),
    DuplicateExprVar(ExprVar),
    BadRecordProj(Type, ExprVar),
    BadApp(Type, Arity),
    BadLam(Type, Arity),
    BadVariant(Type, ExprCon),
    BadMatch(Type),
    BadBranch(Type, ExprCon),
    EmptyMatch,
    TypeAnnsNeeded(Expr),
    NotImplemented(&'static str),
}

impl Module {
    pub fn check(&mut self) -> Result<(), Error> {
        if let Some(name) = find_duplicate(self.type_decls().map(|decl| &decl.name)) {
            return Err(Error::DuplicateTypeDecl(name.clone()));
        }

        let type_syns = self
            .type_decls()
            .map(|decl| (decl.name.clone(), decl.params.len()))
            .collect();
        let type_vars = im::HashSet::new();
        let kind_env = KindEnv {
            type_syns,
            type_vars,
        };
        for type_decl in self.type_decls_mut() {
            type_decl.check(&kind_env)?;
        }
        let funcs = self
            .func_decls_mut()
            .map(|decl| Ok((decl.name.clone(), decl.check_signature(&kind_env)?)))
            .collect::<Result<_, _>>()?;
        let expr_vars = im::HashMap::new();
        let type_env = TypeEnv {
            kind_env,
            funcs,
            expr_vars,
        };
        for decl in self.func_decls_mut() {
            decl.check(&type_env)?;
        }
        Ok(())
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
            body: Type::Fun(
                expr_params.iter().map(|(_, typ)| typ.clone()).collect(),
                Box::new(return_type.clone()),
            ),
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
        env.kind_env.type_vars = type_params.iter().cloned().collect();
        env.expr_vars = expr_params.iter().cloned().collect();
        body.check(env, return_type)?;
        Ok(())
    }
}

impl Type {
    fn check(&mut self, env: &KindEnv) -> Result<(), Error> {
        match self {
            Self::Error => Ok(()),
            Self::Int | Self::Bool => panic!("{:?} in Type.check", self),
            Self::Var(var) => {
                if env.type_vars.contains(var) {
                    Ok(())
                } else if let Some(arity) = env.type_syns.get(var) {
                    if *arity == 0 {
                        *self = Self::SynApp(var.clone(), vec![]);
                        Ok(())
                    } else {
                        Err(Error::KindMismatch {
                            type_var: var.clone(),
                            expected: 0,
                            found: *arity,
                        })
                    }
                } else if let Some(builtin) = BUILTIN_TYPES.get(var) {
                    *self = builtin.clone();
                    Ok(())
                } else {
                    Err(Error::UnknownTypeVar(var.clone()))
                }
            }
            Self::SynApp(var, args) => {
                let num_args = args.len();
                assert!(num_args > 0);
                if env.type_vars.contains(var) {
                    Err(Error::KindMismatch {
                        type_var: var.clone(),
                        expected: num_args,
                        found: 0,
                    })
                } else if let Some(arity) = env.type_syns.get(var) {
                    if *arity == num_args {
                        for arg in args {
                            arg.check(env)?;
                        }
                        Ok(())
                    } else {
                        Err(Error::KindMismatch {
                            type_var: var.clone(),
                            expected: num_args,
                            found: *arity,
                        })
                    }
                } else if BUILTIN_TYPES.contains_key(var) {
                    Err(Error::KindMismatch {
                        type_var: var.clone(),
                        expected: num_args,
                        found: 0,
                    })
                } else {
                    Err(Error::UnknownTypeVar(var.clone()))
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

    pub fn subst(&mut self, mapping: &im::HashMap<&TypeVar, &Type>) -> Result<(), Error> {
        match self {
            Self::Var(var) => {
                if let Some(&typ) = mapping.get(var) {
                    *self = typ.clone();
                    Ok(())
                } else {
                    Err(Error::UnknownTypeVar(var.clone()))
                }
            }
            _ => {
                for child in self.children_mut() {
                    child.subst(mapping)?;
                }
                Ok(())
            }
        }
    }
}

impl TypeScheme {
    pub fn instantiate(&self, var: &ExprVar, types: &Vec<Type>) -> Result<Type, Error> {
        let TypeScheme { params, body } = self;
        let arity = params.len();
        let num_types = types.len();
        if arity == num_types {
            let mut body = body.clone();
            let mapping = params.iter().zip(types.iter()).collect();
            body.subst(&mapping)?;
            Ok(body)
        } else {
            Err(Error::SchemeMismatch {
                expr_var: var.clone(),
                expected: num_types,
                found: arity,
            })
        }
    }
}

impl Expr {
    pub fn check(&mut self, env: &TypeEnv, expected: &Type) -> Result<(), Error> {
        self.infer(env, Some(expected))?;
        Ok(())
    }

    // TODO(MH): Put inferred types in AST.
    fn infer(&mut self, env: &TypeEnv, opt_expected: Option<&Type>) -> Result<Type, Error> {
        match self {
            Self::Error => Ok(Type::Error),
            Self::Var(var) => {
                if let Some(typ) = env.expr_vars.get(var) {
                    self.found_vs_expected(typ.clone(), opt_expected)
                } else if let Some(TypeScheme { params, body }) = env.funcs.get(var) {
                    let arity = params.len();
                    if arity == 0 {
                        *self = Self::FunInst(var.clone(), vec![]);
                        self.found_vs_expected(body.clone(), opt_expected)
                    } else {
                        Err(Error::SchemeMismatch {
                            expr_var: var.clone(),
                            expected: 0,
                            found: arity,
                        })
                    }
                } else {
                    Err(Error::UnknownExprVar(var.clone()))
                }
            }
            Self::Num(_) => self.found_vs_expected(Type::Int, opt_expected),
            Self::Bool(_) => self.found_vs_expected(Type::Bool, opt_expected),
            Self::Lam(params, body) => {
                for (_, opt_typ) in params.iter_mut() {
                    if let Some(typ) = opt_typ {
                        typ.check(&env.kind_env)?;
                    }
                }
                ExprVar::check_unique(params.iter().map(|(name, _)| name))?;
                match opt_expected {
                    None if params
                        .iter()
                        .all(|(_, opt_type_ann)| opt_type_ann.is_some()) =>
                    {
                        let env = &mut env.clone();
                        let mut param_types = Vec::new();
                        for (var, opt_type_ann) in params {
                            let type_ann = opt_type_ann.as_ref().unwrap();
                            env.expr_vars.insert(var.clone(), type_ann.clone());
                            param_types.push(type_ann.clone());
                        }
                        let result = body.infer(env, None)?;
                        Ok(Type::Fun(param_types.clone(), Box::new(result)))
                    }
                    Some(Type::Fun(param_types, result)) if params.len() == param_types.len() => {
                        let env = &mut env.clone();
                        for ((var, opt_type_ann), expected) in params.iter_mut().zip(param_types.iter())
                        {
                            let typ = Expr::Var(var.clone())
                                .found_vs_expected(expected.clone(), opt_type_ann.as_ref())?;
                            if opt_type_ann.is_none() {
                                *opt_type_ann = Some(typ.clone());
                            }
                            env.expr_vars.insert(var.clone(), typ);
                        }
                        let result = body.infer(env, Some(result))?;
                        Ok(Type::Fun(param_types.clone(), Box::new(result)))
                    }
                    None => Err(Error::TypeAnnsNeeded(self.clone())),
                    Some(expected) => Err(Error::BadLam(expected.clone(), params.len())),
                }
            }
            Self::App(fun, args) => {
                let fun_type = fun.infer(env, None)?;
                match fun_type {
                    Type::Fun(params, result) if params.len() == args.len() => {
                        for (arg, typ) in args.iter_mut().zip(params.iter()) {
                            arg.infer(env, Some(typ))?;
                        }
                        self.found_vs_expected(*result, opt_expected)
                    }
                    _ => Err(Error::BadApp(fun_type, args.len())),
                }
            }
            Self::BinOp(lhs, op, rhs) => match op {
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                    lhs.infer(env, Some(&Type::Int))?;
                    rhs.infer(env, Some(&Type::Int))?;
                    self.found_vs_expected(Type::Int, opt_expected)
                }
                OpCode::Equals
                | OpCode::NotEq
                | OpCode::Less
                | OpCode::LessEq
                | OpCode::Greater
                | OpCode::GreaterEq => {
                    let typ = lhs.infer(env, None)?;
                    rhs.infer(env, Some(&typ))?;
                    self.found_vs_expected(Type::Bool, opt_expected)
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
                        expr_var: var.clone(),
                        expected: num_types,
                        found: 0,
                    })
                } else if let Some(scheme) = env.funcs.get(var) {
                    let found = scheme.instantiate(var, types)?;
                    self.found_vs_expected(found, opt_expected)
                } else {
                    Err(Error::UnknownExprVar(var.clone()))
                }
            }
            Self::Let(binder, opt_typ, bindee, body) => {
                if let Some(typ) = opt_typ {
                    typ.check(&env.kind_env)?;
                }
                let typ = bindee.infer(env, opt_typ.as_ref())?;
                if opt_typ.is_none() {
                    *opt_typ = Some(typ.clone());
                }
                let env = &mut env.clone();
                env.expr_vars.insert(binder.clone(), typ);
                body.infer(env, opt_expected)
            }
            Self::If(cond, then, elze) => {
                cond.infer(env, Some(&Type::Bool))?;
                let typ = then.infer(env, opt_expected)?;
                elze.infer(env, Some(&typ))?;
                Ok(typ)
            }
            Self::Record(fields) => {
                let fields = fields
                    .iter_mut()
                    .map(|(name, expr)| Ok((name.clone(), expr.infer(env, None)?)))
                    .collect::<Result<_, _>>()?;
                self.found_vs_expected(Type::Record(fields), opt_expected)
            }
            Self::Proj(record, field) => {
                let mut record_typ = record.infer(env, None)?;
                match &mut record_typ {
                    Type::Record(fields) => {
                        if let Some(field_typ) = find_by_key(fields, field) {
                            self.found_vs_expected(field_typ.clone(), opt_expected)
                        } else {
                            Err(Error::BadRecordProj(record_typ, field.clone()))
                        }
                    }
                    _ => Err(Error::BadRecordProj(record_typ, field.clone())),
                }
            }
            Self::Variant(con, arg) => match opt_expected {
                Some(Type::Variant(cons)) => {
                    if let Some(arg_typ) = find_by_key(cons, con) {
                        arg.infer(env, Some(arg_typ))?;
                        Ok(Type::Variant(cons.clone()))
                    } else {
                        Err(Error::BadVariant(Type::Variant(cons.clone()), con.clone()))
                    }
                }
                Some(expected) => Err(Error::BadVariant(expected.clone(), con.clone())),
                None => Err(Error::TypeAnnsNeeded(self.clone())),
            },
            Self::Match(scrut, branches) => {
                let scrut_typ = scrut.infer(env, None)?;
                match scrut_typ {
                    Type::Variant(cons) => {
                        if let Some((first, rest)) = branches.split_first_mut() {
                            let rhs_typ = first.infer(env, &cons, opt_expected)?;
                            for other in rest {
                                other.infer(env, &cons, Some(&rhs_typ))?;
                            }
                            Ok(rhs_typ)
                        } else {
                            Err(Error::EmptyMatch)
                        }
                    }
                    _ => Err(Error::BadMatch(scrut_typ)),
                }
            }
        }
    }

    fn found_vs_expected(&self, found: Type, opt_expected: Option<&Type>) -> Result<Type, Error> {
        match opt_expected {
            None => Ok(found),
            Some(expected) => {
                if found == *expected {
                    Ok(found)
                } else {
                    Err(Error::TypeMismatch {
                        expr: self.clone(),
                        expected: expected.clone(),
                        found,
                    })
                }
            }
        }
    }
}

impl Branch {
    fn infer(
        &mut self,
        env: &TypeEnv,
        cons: &Vec<(ExprCon, Type)>,
        opt_expected: Option<&Type>,
    ) -> Result<Type, Error> {
        let Branch {
            con,
            var: opt_var,
            rhs,
        } = self;
        if let Some(arg_type) = find_by_key(cons, con) {
            if let Some(var) = opt_var {
                let env = &mut env.clone();
                env.expr_vars.insert(var.clone(), arg_type.clone());
                rhs.infer(env, opt_expected)
            } else {
                rhs.infer(env, opt_expected)
            }
        } else {
            Err(Error::BadBranch(Type::Variant(cons.clone()), con.clone()))
        }
    }
}

impl TypeVar {
    fn check_unique<'a, I: Iterator<Item = &'a TypeVar>>(iter: I) -> Result<(), Error> {
        if let Some(dup) = find_duplicate(iter) {
            Err(Error::DuplicateTypeVar(dup.clone()))
        } else {
            Ok(())
        }
    }
}

impl ExprVar {
    fn check_unique<'a, I: Iterator<Item = &'a ExprVar>>(iter: I) -> Result<(), Error> {
        if let Some(dup) = find_duplicate(iter) {
            Err(Error::DuplicateExprVar(dup.clone()))
        } else {
            Ok(())
        }
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
