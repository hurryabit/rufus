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
    KindMismatch {
        type_var: TypeVar,
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
    fn check(&mut self, env: &Env) -> Result<(), Error> {
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
