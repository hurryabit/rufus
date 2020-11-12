use serde::{Serialize, Serializer};
use std::rc::Rc;

use crate::syntax;
use syntax::{ExprCon, ExprVar, TypeVar};

type SynType = syntax::Type;

#[derive(Debug, Serialize)]
pub enum Type<T = RcType> {
    Error,
    Var(TypeVar),
    SynApp(TypeVar, Vec<T>),
    Int,
    Bool,
    Fun(Vec<T>, T),
    Record(Vec<(ExprVar, T)>),
    Variant(Vec<(ExprCon, T)>),
}

#[derive(Clone, Debug)]
pub struct RcType(Rc<Type>);

#[derive(Debug, Serialize)]
pub struct TypeScheme {
    pub params: Vec<TypeVar>,
    pub body: RcType,
}

type TypeDefs = std::collections::HashMap<TypeVar, TypeScheme>;

impl RcType {
    pub fn new(typ: Type) -> Self {
        Self(Rc::new(typ))
    }

    pub fn from_syntax(syntax: &SynType) -> Self {
        Self::new(Type::from_syntax(syntax))
    }

    pub fn to_syntax(&self) -> SynType {
        Type::to_syntax(&*self)
    }

    pub fn subst(&self, mapping: &std::collections::HashMap<&TypeVar, &RcType>) -> RcType {
        match &**self {
            Type::Var(var) => {
                let &typ = mapping
                    .get(&var)
                    .expect("subst: free vars must be a subset of the substitution's domain");
                typ.clone()
            }
            typ => Self::new(typ.map(|child| child.subst(mapping))),
        }
    }

    pub fn weak_normalize(&self, type_defs: &TypeDefs) -> Self {
        let mut typ = self.clone();
        while let Type::SynApp(var, args) = &*typ {
            let scheme = type_defs.get(&var).unwrap();
            typ = scheme.instantiate(&args);
        }
        typ
    }

    pub fn equiv(&self, expected: &RcType, type_defs: &TypeDefs) -> bool {
        use Type::*;
        match (&**self, &**expected) {
            (SynApp(var1, args1), SynApp(var2, args2)) if var1 == var2 => {
                assert_eq!(args1.len(), args2.len());
                args1
                    .iter()
                    .zip(args2.iter())
                    .all(|(arg1, arg2)| arg1.equiv(arg2, type_defs))
            }
            _ => match (
                &*self.weak_normalize(type_defs),
                &*expected.weak_normalize(type_defs),
            ) {
                (SynApp(_, _), _) | (_, SynApp(_, _)) => {
                    panic!("IMPOSSIBLE: Type::SynApp after Type::weak_normalize")
                }
                (Error, _) | (_, Error) => true,
                (Var(var1), Var(var2)) => var1 == var2,
                (Int, Int) => true,
                (Bool, Bool) => true,
                (Fun(params1, result1), Fun(params2, result2)) => {
                    params1.len() == params2.len()
                        && params1
                            .iter()
                            .zip(params2.iter())
                            .all(|(param1, param2)| param1.equiv(param2, type_defs))
                        && result1.equiv(result2, type_defs)
                }
                (Record(fields1), Record(fields2)) => {
                    same_keys(&fields1, &fields2)
                        && fields1
                            .iter()
                            .zip(fields2.iter())
                            .all(|((_, typ1), (_, typ2))| typ1.equiv(typ2, type_defs))
                }
                (Variant(constrs1), Variant(constrs2)) => {
                    same_keys(&constrs1, &constrs2)
                        && constrs1
                            .iter()
                            .zip(constrs2.iter())
                            .all(|((_, typ1), (_, typ2))| typ1.equiv(typ2, type_defs))
                }
                (Var(_), _)
                | (Int, _)
                | (Bool, _)
                | (Fun(_, _), _)
                | (Record(_), _)
                | (Variant(_), _) => false,
            },
        }
    }
}

impl TypeScheme {
    /// Instantiate a type scheme with the given types. Assumes that the
    /// number of parameters of the scheme and the number of given types match.
    pub fn instantiate(&self, types: &Vec<RcType>) -> RcType {
        let Self { params, body } = self;
        assert_eq!(params.len(), types.len());
        let mapping = params.iter().zip(types.iter()).collect();
        body.subst(&mapping)
    }
}

impl Type {
    pub fn from_syntax(syntax: &SynType) -> Self {
        match syntax {
            SynType::Error => Type::Error,
            SynType::Var(var) => Type::Var(*var),
            SynType::SynApp(var, args) => {
                let args = args.iter().map(RcType::from_syntax).collect();
                Type::SynApp(*var, args)
            }
            SynType::Int => Type::Int,
            SynType::Bool => Type::Bool,
            SynType::Fun(params, result) => {
                let params = params.iter().map(RcType::from_syntax).collect();
                let result = RcType::from_syntax(result);
                Type::Fun(params, result)
            }
            SynType::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, typ)| (*name, RcType::from_syntax(typ)))
                    .collect();
                Type::Record(fields)
            }
            SynType::Variant(constrs) => {
                let constrs = constrs
                    .iter()
                    .map(|(name, typ)| (*name, RcType::from_syntax(typ)))
                    .collect();
                Type::Variant(constrs)
            }
        }
    }

    pub fn to_syntax(&self) -> SynType {
        match self {
            Type::Error => SynType::Error,
            Type::Var(var) => SynType::Var(*var),
            Type::SynApp(var, args) => {
                let args = args.iter().map(RcType::to_syntax).collect();
                SynType::SynApp(*var, args)
            }
            Type::Int => SynType::Int,
            Type::Bool => SynType::Bool,
            Type::Fun(params, result) => {
                let params = params.iter().map(RcType::to_syntax).collect();
                let result = Box::new(RcType::to_syntax(result));
                SynType::Fun(params, result)
            }
            Type::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, typ)| (*name, typ.to_syntax()))
                    .collect();
                SynType::Record(fields)
            }
            Type::Variant(constrs) => {
                let constrs = constrs
                    .iter()
                    .map(|(name, typ)| (*name, typ.to_syntax()))
                    .collect();
                SynType::Variant(constrs)
            }
        }
    }
}

impl<T> Type<T> {
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut T> {
        use genawaiter::{rc::gen, yield_};
        use Type::*;
        gen!({
            match self {
                Error => {}
                Var(_) | Int | Bool => {}
                SynApp(syn, args) => {
                    let _: &TypeVar = syn; // We want this to break if change the type of `syn`.
                    for arg in args {
                        yield_!(arg);
                    }
                }
                Fun(params, result) => {
                    for param in params {
                        yield_!(param);
                    }
                    yield_!(result);
                }
                Record(fields) => {
                    for (_name, typ) in fields {
                        yield_!(typ);
                    }
                }
                Variant(constrs) => {
                    for (_name, typ) in constrs {
                        yield_!(typ);
                    }
                }
            }
        })
        .into_iter()
    }

    pub fn map<U, F>(&self, f: F) -> Type<U>
    where
        F: Fn(&T) -> U,
    {
        use Type::*;
        match self {
            Error => Error,
            Var(var) => Var(*var),
            SynApp(var, args) => {
                let args = args.iter().map(f).collect();
                SynApp(*var, args)
            }
            Int => Int,
            Bool => Bool,
            Fun(params, result) => {
                let params = params.iter().map(&f).collect();
                let result = f(result);
                Fun(params, result)
            }
            Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, child)| (*name, f(child)))
                    .collect();
                Record(fields)
            }
            Variant(constrs) => {
                let constrs = constrs
                    .iter()
                    .map(|(name, child)| (*name, f(child)))
                    .collect();
                Variant(constrs)
            }
        }
    }
}

impl std::ops::Deref for RcType {
    type Target = Type;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl AsRef<Type> for RcType {
    fn as_ref(&self) -> &Type {
        self.0.as_ref()
    }
}

impl Serialize for RcType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

pub fn same_keys<'a, K: Eq, V>(vec1: &'a [(K, V)], vec2: &'a [(K, V)]) -> bool {
    vec1.len() == vec2.len()
        && vec1
            .iter()
            .zip(vec2.iter())
            .all(|((k1, _), (k2, _))| k1 == k2)
}