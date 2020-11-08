#[macro_use]
extern crate lalrpop_util;

pub mod syntax;
lalrpop_mod!(
    #[allow(clippy::all)]
    pub parser
);


#[cfg(test)]
mod tests {
    mod parser {
        use crate::*;
        use syntax::*;

        #[test]
        fn types_positive() {
            use syntax::Type::*;
            let parser = parser::TypeParser::new();

            let cases = &[
                ("Int", Int),
                ("Bool", Bool),
                ("a", Var(TypeVar::new("a"))),
                ("A", Synonym(TypeCon::new("A"))),
                ("() -> Int", Fun(vec![], Box::new(Int))),
                ("(Int) -> Int", Fun(vec![Int], Box::new(Int))),
                ("(Int,) -> Int", Fun(vec![Int], Box::new(Int))),
                ("A<Int>", App(TypeCon::new("A"), vec![Int])),
                ("A<Int,>", App(TypeCon::new("A"), vec![Int])),
                ("{}", Record(vec![])),
                ("{a: Int}", Record(vec![(ExprVar::new("a"), Int)])),
                ("{a: Int,}", Record(vec![(ExprVar::new("a"), Int)])),
                ("[A | B(Int)]", Variant(vec![(ExprCon::new("A"), None), (ExprCon::new("B"), Some(Int))])),
                // TODO(MH): We want to allow an optional leading "|" rather
                // than a trailing one.
                ("[A | B(Int) |]", Variant(vec![(ExprCon::new("A"), None), (ExprCon::new("B"), Some(Int))])),
            ];

            for (input, expected) in cases {
                assert_eq!(parser.parse(input).as_ref(), Ok(expected))
            }
        }

        #[test]
        fn types_negative() {
            let parser = parser::TypeParser::new();

            let cases = &[
                // These makes no sense.
                "(,) -> Int",
                "A<>",
                "{,}",
                // This would not kind check but might give bad error messages
                // because `Int` cannot be resolved.
                "Int<Bool>",
                // We don't have higher-kinded type variables.
                "a<Int>",
                // We don't support empty variants.
                "[]",
            ];

            for input in cases {
                assert!(parser.parse(input).is_err());
            }
        }
    }
}
