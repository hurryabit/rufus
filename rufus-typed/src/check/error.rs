use super::types::*;
use super::Arity;
use crate::syntax;
use std::fmt;
use syntax::{ExprCon, ExprVar, Located, Span, TypeVar};

#[derive(Debug)]
pub enum Error<Pos = usize> {
    UnknownTypeVar(TypeVar),
    UnknownExprVar(ExprVar),
    UnexpectedGeneric(TypeVar, Arity),
    GenericTypeArityMismatch {
        type_var: TypeVar,
        expected: Arity,
        found: Arity,
    },
    GenericFuncArityMismatch {
        expr_var: ExprVar,
        expected: Arity,
        found: Arity,
    },
    TypeMismatch {
        expected: RcType,
        found: RcType,
    },
    ParamTypeMismatch {
        param: ExprVar,
        expected: RcType,
        found: RcType,
    },
    DuplicateTypeVar {
        var: TypeVar,
        original: Span<Pos>,
    },
    DuplicateTypeDecl {
        var: TypeVar,
        original: Span<Pos>,
    },
    DuplicateExprVar {
        var: ExprVar,
        original: Span<Pos>,
    },
    BadApp {
        func: Option<ExprVar>,
        func_type: RcType,
        num_args: Arity,
    },
    BadRecordProj {
        record_type: RcType,
        field: ExprVar,
    },
    BadLam(RcType, Arity),
    BadVariantConstr(RcType, ExprCon),
    UnexpectedVariantType(RcType, ExprCon),
    BadMatch(RcType),
    BadBranch(RcType, ExprCon),
    EmptyMatch,
    TypeAnnsNeeded,
}

pub type LError<Pos = usize> = Located<Error<Pos>, Pos>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn plural(n: Arity) -> &'static str {
            if n == 1 {
                ""
            } else {
                "s"
            }
        }

        use Error::*;
        match self {
            UnknownTypeVar(var) => write!(f, "Undeclared type variable `{}`.", var),
            UnknownExprVar(var) => write!(f, "Undeclared variable `{}`.", var),
            UnexpectedGeneric(var, _arity) => {
                write!(f, "Expected a type but found the generic type `{}`.", var)
            }
            GenericTypeArityMismatch {
                type_var,
                expected: 0,
                found,
            }  => write!(
                f,
                "Type `{}` is not a generic type but is applied to {} type argument{}.",
                type_var, found, plural(*found)
            ),
            GenericTypeArityMismatch {
                type_var,
                expected,
                found,
            } => write!(
                f,
                "Generic type `{}` expects {} type argument{} but is applied to {} type argument{}.",
                type_var, expected, plural(*expected), found, plural(*found)
            ),
            GenericFuncArityMismatch {
                expr_var,
                expected: 0,
                found,
            } => write!(
                f,
                "`{}` is not a generic function but is applied to {} type argument{}.",
                expr_var, found, plural(*found)
            ),
            GenericFuncArityMismatch {
                expr_var,
                expected,
                found,
            } => write!(
                f,
                "Generic function `{}` expects {} type argument{} but is applied to {} type argument{}.",
                expr_var, expected, plural(*expected), found, plural(*found)
            ),
            TypeMismatch { expected, found } => write!(
                f,
                "Expected an expression of type `{}` but found an expression of type `{}`.",
                expected, found,
            ),
            ParamTypeMismatch { param, expected, found } => write!(
                f,
                "Expected parameter `{}` to have type `{}` but found a type annotation `{}`.",
                param, expected, found,
            ),
            DuplicateTypeVar { var, original: _ } => write!(f, "Duplicate type variable `{}`.", var),
            DuplicateTypeDecl { var, original: _ } => {
                write!(f, "Duplicate definition of type `{}`.", var)
            }
            DuplicateExprVar { var, original: _ } => write!(f, "Duplicate variable `{}`.", var),
            BadApp {
                func: Some(func),
                func_type,
                num_args,
            } => write!(
                f,
                "`{}` cannot be applied to {} argument{} because it has has type `{}`.",
                func, num_args, plural(*num_args), func_type
            ),
            BadApp {
                func: None,
                func_type,
                num_args,
            } => write!(
                f,
                "Expressions of type `{}` cannot be applied to {} argument{}.",
                func_type, num_args, plural(*num_args)
            ),
            BadRecordProj { record_type, field } => write!(
                f,
                "Expression of type `{}` do not contain a field named `{}`.",
                record_type, field
            ),
            BadLam(expected, arity) => write!(
                f,
                "Expected an expression of type `{}` but found a lambda with {} parameter{}.",
                expected, arity, plural(*arity)
            ),
            BadVariantConstr(expected, con) => write!(
                f,
                "`{}` is not a possible constructor for variant type `{}`.",
                con, expected
            ),
            UnexpectedVariantType(expected, _con) => write!(
                f,
                "Expected an expression of type `{}` but found variant constructor.",
                expected
            ),
            EmptyMatch => write!(f, "Match expressions must have at least one branch."),
            BadMatch(scrut_type) => {
                write!(f, "Cannot match on expressions of type `{}`.", scrut_type)
            }
            BadBranch(scrut_type, con) => write!(
                f,
                "`{}` is not a possible constructor for variant type `{}`.",
                con, scrut_type
            ),
            TypeAnnsNeeded => write!(f, "Cannot infer the type of the expression. Further type annotations are required."),
        }
    }
}
