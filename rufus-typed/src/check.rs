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
pub struct Env {
    type_syns: im::HashMap<TypeVar, Arity>,
    type_vars: im::HashSet<TypeVar>,
}

#[derive(Debug)]
pub enum Error {
    UnknownTypeVar(TypeVar),
    ExpectedTypeFoundTypeCon(Type),
    WrongNumberOfTypeArgs {
        typ: Type,
        expected: Arity,
        found: Arity,
    },
    DuplicateTypeVar(TypeVar),
    DuplicateTypeDecl(TypeVar),
}

impl Module {
    pub fn check(&mut self) -> Result<(), Error> {
        if let Some(name) = find_duplicate(self.type_decls().map(|decl| &decl.name)) {
            return Err(Error::DuplicateTypeDecl(name.clone()));
        }

        let type_syns = self
            .type_decls()
            .map(|decl| (decl.name.clone(), decl.params.len()))
            .collect::<im::HashMap<_, _>>();
        let type_vars = im::HashSet::new();
        let env = Env {
            type_syns,
            type_vars,
        };
        for type_decl in self.type_decls_mut() {
            type_decl.check(&env)?;
        }
        Ok(())
    }
}

// impl Decl {
//     pub fn check(&self, env: &Env) -> Result<Self, Error> {
//         Ok(match self {
//             Decl::Type(decl) => Decl::Type(decl.check(env)?),
//             Decl::Func(decl) => Decl::Func(decl.clone()),
//         })
//     }
// }

impl TypeDecl {
    pub fn check(&mut self, env: &Env) -> Result<(), Error> {
        let Self { name: _, params, body } = self;
        TypeVar::check_unique(params.iter())?;
        let mut env = env.clone();
        env.type_vars.extend(params.iter().cloned());
        body.check(&mut env)
    }
}

impl Type {
    pub fn check(&mut self, env: &Env) -> Result<(), Error> {
        let arity = self.infer(env)?;
        if arity == 0 {
            Ok(())
        } else {
            Err(Error::ExpectedTypeFoundTypeCon(self.clone()))
        }
    }

    fn infer(&mut self, env: &Env) -> Result<Arity, Error> {
        match self {
            Self::Error => Ok(0),
            Self::Syn(_) | Self::Int | Self::Bool => panic!("{:?} in Type.check", self),
            Self::Var(name) => {
                if env.type_vars.contains(name) {
                    Ok(0)
                } else if let Some(arity) = env.type_syns.get(name) {
                    let syn = Self::Syn(name.clone());
                    *self = syn;
                    Ok(*arity)
                } else if let Some(builtin) = BUILTIN_TYPES.get(name) {
                    *self = builtin.clone();
                    Ok(0)
                } else {
                    Err(Error::UnknownTypeVar(name.clone()))
                }
            }
            Self::App(fun, args) => {
                let num_args = args.len();
                assert!(num_args > 0);
                let arity = fun.infer(&env)?;
                if num_args != arity {
                    return Err(Error::WrongNumberOfTypeArgs {
                        typ: self.clone(),
                        expected: arity,
                        found: num_args,
                    });
                }
                for arg in args {
                    arg.check(env)?;
                }
                Ok(0)
            }
            Self::Fun(_, _) | Self::Record(_) | Self::Variant(_) => {
                for child in self.children_mut() {
                    child.check(env)?;
                }
                Ok(0)
            }
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
