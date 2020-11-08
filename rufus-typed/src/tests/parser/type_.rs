use crate::*;
use syntax::*;

fn int() -> Type {
    Type::Var(TypeVar::new("Int"))
}

fn bool() -> Type {
    Type::Var(TypeVar::new("Bool"))
}

#[test]
fn types_positive() {
    use syntax::Type::*;
    let parser = parser::TypeParser::new();

    let cases = &[
        ("A", Var(TypeVar::new("A"))),
        ("() -> Int", Fun(vec![], Box::new(int()))),
        ("(Int) -> Int", Fun(vec![int()], Box::new(int()))),
        ("(Int,) -> Int", Fun(vec![int()], Box::new(int()))),
        ("A<Int>", App(TypeVar::new("A"), vec![int()])),
        ("A<Int,>", App(TypeVar::new("A"), vec![int()])),
        ("A<Int,Bool>", App(TypeVar::new("A"), vec![int(), bool()])),
        ("{}", Record(vec![])),
        ("{a: Int}", Record(vec![(ExprVar::new("a"), int())])),
        ("{a: Int,}", Record(vec![(ExprVar::new("a"), int())])),
        (
            "[A | B(Int)]",
            Variant(vec![
                (ExprCon::new("A"), None),
                (ExprCon::new("B"), Some(int())),
            ]),
        ),
        (
            "[Int(Int)]",
            Variant(vec![(ExprCon::new("Int"), Some(int()))]),
        ),
        (
            "[Bool(Bool)]",
            Variant(vec![(ExprCon::new("Bool"), Some(bool()))]),
        ),
        // TODO(MH): We want to allow an optional leading "|" rather
        // than a trailing one.
        (
            "[A | B(Int) |]",
            Variant(vec![
                (ExprCon::new("A"), None),
                (ExprCon::new("B"), Some(int())),
            ]),
        ),
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
        // We don't support empty variants.
        "[]",
    ];

    for input in cases {
        assert!(parser.parse(input).is_err());
    }
}
