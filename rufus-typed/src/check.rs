use crate::syntax::*;
use std::hash::Hash;

type Arity = usize;

#[derive(Clone)]
pub struct KindEnv {
    builtin_types: im::HashMap<TypeVar, Type>,
    types: im::HashMap<TypeVar, TypeScheme>,
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
        let mut builtin_types = im::HashMap::new();
        builtin_types.insert(TypeVar::new("Int"), Type::Int);
        builtin_types.insert(TypeVar::new("Bool"), Type::Bool);

        if let Some(name) = find_duplicate(self.type_decls().map(|decl| &decl.name)) {
            return Err(Error::DuplicateTypeDecl(*name));
        }

        let types = self.types();
        let type_vars = im::HashSet::new();
        let mut kind_env = KindEnv {
            builtin_types,
            types,
            type_vars,
        };
        for type_decl in self.type_decls_mut() {
            type_decl.check(&kind_env)?;
        }
        kind_env.types = self.types();
        let funcs = self
            .func_decls_mut()
            .map(|decl| Ok((decl.name, decl.check_signature(&kind_env)?)))
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

    fn types(&self) -> im::HashMap<TypeVar, TypeScheme> {
        self.type_decls()
            .map(|TypeDecl { name, params, body }| {
                (
                    *name,
                    TypeScheme {
                        params: params.clone(),
                        body: body.clone(),
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
        body.check(env, return_type.clone())?;
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

    pub fn subst(&mut self, mapping: &im::HashMap<&TypeVar, &Type>) -> () {
        match self {
            Self::Var(var) => {
                let &typ = mapping.get(var).unwrap();
                *self = typ.clone();
            }
            _ => {
                for child in self.children_mut() {
                    child.subst(mapping);
                }
            }
        }
    }

    pub fn weak_normalize(self, env: &TypeEnv) -> Type {
        let mut typ = self;
        while let Type::SynApp(var, args) = typ {
            let scheme = env.kind_env.types.get(&var).unwrap();
            typ = scheme.instantiate(&args);
        }
        typ
    }
}

impl TypeScheme {
    pub fn instantiate(&self, types: &Vec<Type>) -> Type {
        let TypeScheme { params, body } = self;
        assert_eq!(params.len(), types.len());
        let mut body = body.clone();
        let mapping = params.iter().zip(types.iter()).collect();
        body.subst(&mapping);
        body
    }
}

impl Expr {
    pub fn check(&mut self, env: &TypeEnv, expected: Type) -> Result<(), Error> {
        self.infer(env, Some(expected))?;
        Ok(())
    }

    fn infer(&mut self, env: &TypeEnv, opt_expected: Option<Type>) -> Result<Type, Error> {
        match self {
            Self::Error => Ok(Type::Error),
            Self::Var(var) => {
                if let Some(typ) = env.expr_vars.get(var) {
                    self.found_vs_opt_expected(env, typ.clone(), opt_expected)
                } else if let Some(TypeScheme { params, body }) = env.funcs.get(var) {
                    let arity = params.len();
                    if arity == 0 {
                        *self = Self::FunInst(*var, vec![]);
                        self.found_vs_opt_expected(env, body.clone(), opt_expected)
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
            Self::Num(_) => self.found_vs_opt_expected(env, Type::Int, opt_expected),
            Self::Bool(_) => self.found_vs_opt_expected(env, Type::Bool, opt_expected),
            Self::Lam(params, body) => {
                for (_, opt_typ) in params.iter_mut() {
                    if let Some(typ) = opt_typ {
                        typ.check(&env.kind_env)?;
                    }
                }
                ExprVar::check_unique(params.iter().map(|(name, _)| name))?;
                match opt_expected {
                    None => {
                        if params
                            .iter()
                            .all(|(_, opt_type_ann)| opt_type_ann.is_some())
                        {
                            let env = &mut env.clone();
                            let mut param_types = Vec::new();
                            for (var, opt_type_ann) in params {
                                // TODO(MH): Use `collect::<Option<_>>`.
                                let type_ann = opt_type_ann.as_ref().unwrap();
                                env.expr_vars.insert(*var, type_ann.clone());
                                param_types.push(type_ann.clone());
                            }
                            let result = body.infer(env, None)?;
                            Ok(Type::Fun(param_types.clone(), Box::new(result)))
                        } else {
                            Err(Error::TypeAnnsNeeded(self.clone()))
                        }
                    }
                    Some(expected) => match expected.weak_normalize(env) {
                        Type::Fun(param_types, result) if params.len() == param_types.len() => {
                            let env = &mut env.clone();
                            // TODO(MH): Remove some cloning.
                            for ((var, opt_type_ann), expected) in
                                params.iter_mut().zip(param_types.iter())
                            {
                                let typ = Expr::Var(*var).found_vs_opt_expected(
                                    env,
                                    expected.clone(),
                                    opt_type_ann.clone(),
                                )?;
                                if opt_type_ann.is_none() {
                                    *opt_type_ann = Some(typ.clone());
                                }
                                env.expr_vars.insert(*var, typ);
                            }
                            let result = body.infer(env, Some(*result))?;
                            Ok(Type::Fun(param_types.clone(), Box::new(result)))
                        }
                        expected => Err(Error::BadLam(expected, params.len())),
                    },
                }
            }
            Self::App(fun, args) => {
                let fun_type = fun.infer(env, None)?;
                match fun_type.weak_normalize(env) {
                    Type::Fun(params, result) if params.len() == args.len() => {
                        for (arg, typ) in args.iter_mut().zip(params.into_iter()) {
                            arg.infer(env, Some(typ))?;
                        }
                        self.found_vs_opt_expected(env, *result, opt_expected)
                    }
                    fun_type => Err(Error::BadApp(fun_type, args.len())),
                }
            }
            Self::BinOp(lhs, op, rhs) => match op {
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                    lhs.infer(env, Some(Type::Int))?;
                    rhs.infer(env, Some(Type::Int))?;
                    self.found_vs_opt_expected(env, Type::Int, opt_expected)
                }
                OpCode::Equals
                | OpCode::NotEq
                | OpCode::Less
                | OpCode::LessEq
                | OpCode::Greater
                | OpCode::GreaterEq => {
                    let typ = lhs.infer(env, None)?;
                    rhs.infer(env, Some(typ))?;
                    self.found_vs_opt_expected(env, Type::Bool, opt_expected)
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
                        let found = scheme.instantiate(types);
                        self.found_vs_opt_expected(env, found, opt_expected)
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
            Self::Let(binder, opt_typ, bindee, body) => {
                if let Some(typ) = opt_typ {
                    typ.check(&env.kind_env)?;
                }
                let typ = bindee.infer(env, std::mem::take(opt_typ))?;
                *opt_typ = Some(typ.clone());
                let env = &mut env.clone();
                env.expr_vars.insert(binder.clone(), typ);
                body.infer(env, opt_expected)
            }
            Self::If(cond, then, elze) => {
                cond.infer(env, Some(Type::Bool))?;
                let typ = then.infer(env, opt_expected)?;
                elze.infer(env, Some(typ))
            }
            Self::Record(fields) => {
                let fields = fields
                    .iter_mut()
                    .map(|(name, expr)| Ok((*name, expr.infer(env, None)?)))
                    .collect::<Result<_, _>>()?;
                self.found_vs_opt_expected(env, Type::Record(fields), opt_expected)
            }
            Self::Proj(record, field) => {
                let record_typ = record.infer(env, None)?;
                match record_typ.weak_normalize(env) {
                    Type::Record(fields) => {
                        if let Some(field_typ) = find_by_key(&fields, field) {
                            self.found_vs_opt_expected(env, field_typ.clone(), opt_expected)
                        } else {
                            Err(Error::BadRecordProj(Type::Record(fields), *field))
                        }
                    }
                    record_typ => Err(Error::BadRecordProj(record_typ, *field)),
                }
            }
            Self::Variant(con, arg) => match opt_expected {
                None => Err(Error::TypeAnnsNeeded(self.clone())),
                Some(expected) => match expected.weak_normalize(env) {
                    Type::Variant(cons) => {
                        if let Some(arg_typ) = find_by_key(&cons, con) {
                            arg.infer(env, Some(arg_typ.clone()))?;
                            Ok(Type::Variant(cons))
                        } else {
                            Err(Error::BadVariant(Type::Variant(cons), *con))
                        }
                    }
                    expected => Err(Error::BadVariant(expected, *con)),
                },
            },
            Self::Match(scrut, branches) => {
                let scrut_typ = scrut.infer(env, None)?;
                match scrut_typ.weak_normalize(env) {
                    Type::Variant(cons) => {
                        if let Some((first, rest)) = branches.split_first_mut() {
                            let mut rhs_typ = first.infer(env, &cons, opt_expected)?;
                            for other in rest {
                                rhs_typ = other.infer(env, &cons, Some(rhs_typ))?;
                            }
                            Ok(rhs_typ)
                        } else {
                            Err(Error::EmptyMatch)
                        }
                    }
                    scrut_typ => Err(Error::BadMatch(scrut_typ)),
                }
            }
        }
    }

    fn found_vs_opt_expected(
        &self,
        env: &TypeEnv,
        found: Type,
        opt_expected: Option<Type>,
    ) -> Result<Type, Error> {
        match opt_expected {
            None => Ok(found),
            Some(expected) => self.found_vs_expected(env, found, expected),
        }
    }

    fn found_vs_expected(&self, env: &TypeEnv, found: Type, expected: Type) -> Result<Type, Error> {
        match (found, expected) {
            (Type::SynApp(var1, args1), Type::SynApp(var2, args2)) if var1 == var2 => {
                assert_eq!(args1.len(), args2.len());
                let args = args1
                    .into_iter()
                    .zip(args2.into_iter())
                    .map(|(arg1, arg2)| Ok(self.found_vs_expected(env, arg1, arg2)?))
                    .collect::<Result<_, _>>()?;
                Ok(Type::SynApp(var1, args))
            }
            (found, expected) => match (found.weak_normalize(env), expected.weak_normalize(env)) {
                (Type::SynApp(_, _), _) | (_, Type::SynApp(_, _)) => {
                    panic!("IMPOSSIBLE: Type::SynApp after Type::weak_normalize")
                }
                (Type::Error, Type::Error) => Ok(Type::Error),
                (Type::Error, typ) | (typ, Type::Error) => Ok(typ),
                (Type::Var(var1), Type::Var(var2)) if var1 == var2 => Ok(Type::Var(var1)),
                (Type::Int, Type::Int) => Ok(Type::Int),
                (Type::Bool, Type::Bool) => Ok(Type::Bool),
                (Type::Fun(params1, result1), Type::Fun(params2, result2))
                    if params1.len() == params2.len() =>
                {
                    let params = params1
                        .into_iter()
                        .zip(params2.into_iter())
                        .map(|(param1, param2)| Ok(self.found_vs_expected(env, param1, param2)?))
                        .collect::<Result<_, _>>()?;
                    let result = self.found_vs_expected(env, *result1, *result2)?;
                    Ok(Type::Fun(params, Box::new(result)))
                }
                (Type::Record(fields1), Type::Record(fields2)) if same_keys(&fields1, &fields2) => {
                    let fields = fields1
                        .into_iter()
                        .zip(fields2.into_iter())
                        .map(|((name, typ1), (_, typ2))| {
                            let typ = self.found_vs_expected(env, typ1, typ2)?;
                            Ok((name, typ))
                        })
                        .collect::<Result<_, _>>()?;
                    Ok(Type::Record(fields))
                }
                (Type::Variant(constrs1), Type::Variant(constrs2))
                    if same_keys(&constrs1, &constrs2) =>
                {
                    let constrs = constrs1
                        .into_iter()
                        .zip(constrs2.into_iter())
                        .map(|((name, typ1), (_, typ2))| {
                            let typ = self.found_vs_expected(env, typ1, typ2)?;
                            Ok((name, typ))
                        })
                        .collect::<Result<_, _>>()?;
                    Ok(Type::Variant(constrs))
                }
                (found @ Type::Var(_), expected)
                | (found @ Type::Int, expected)
                | (found @ Type::Bool, expected)
                | (found @ Type::Fun(_, _), expected)
                | (found @ Type::Record(_), expected)
                | (found @ Type::Variant(_), expected) => Err(Error::TypeMismatch {
                    expr: self.clone(),
                    expected,
                    found,
                }),
            },
        }
    }
}

impl Branch {
    fn infer(
        &mut self,
        env: &TypeEnv,
        cons: &Vec<(ExprCon, Type)>,
        opt_expected: Option<Type>,
    ) -> Result<Type, Error> {
        let Branch {
            con,
            var: opt_var,
            rhs,
        } = self;
        if let Some(arg_type) = find_by_key(cons, con) {
            if let Some(var) = opt_var {
                let env = &mut env.clone();
                env.expr_vars.insert(*var, arg_type.clone());
                rhs.infer(env, opt_expected)
            } else {
                rhs.infer(env, opt_expected)
            }
        } else {
            Err(Error::BadBranch(Type::Variant(cons.clone()), *con))
        }
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

fn same_keys<'a, K: Eq, V>(vec1: &'a [(K, V)], vec2: &'a [(K, V)]) -> bool {
    vec1.len() == vec2.len()
        && vec1
            .iter()
            .zip(vec2.iter())
            .all(|((k1, _), (k2, _))| k1 == k2)
}
