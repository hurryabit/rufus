use crate::*;
use syntax::*;

use lalrpop_util::ParseError;

fn parse_err(input: &'static str) -> (Option<Type>, Vec<ParseError<usize, parser::Token<'static>, &'static str>>) {
    let parser = parser::TypeParser::new();
    let mut errors = Vec::new();
    let result = parser.parse(&mut errors, input);
    assert!(!errors.is_empty() || result.is_err());
    let mut errors = errors
        .into_iter()
        .map(|error_recovery| error_recovery.error)
        .collect::<Vec<_>>();
    match result {
      Ok(expr) => (Some(expr), errors),
      Err(error) => {
        errors.push(error);
        (None, errors)
      }
    }
}

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
        ("A<Int>", Type::var_app(TypeVar::new("A"), vec![int()])),
        ("A<Int,>", Type::var_app(TypeVar::new("A"), vec![int()])),
        ("A<Int,Bool>", Type::var_app(TypeVar::new("A"), vec![int(), bool()])),
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
        let mut errors = Vec::new();
        assert_eq!(parser.parse(&mut errors, input).as_ref(), Ok(expected));
        assert_eq!(errors, vec![]);
    }
}

#[test]
fn func_type_zero_params_one_comma() {
    insta::assert_debug_snapshot!(parse_err("(,) -> Int"), @r###"
    (
        Some(
            Fun(
                [
                    Error,
                ],
                Var(
                    t#Int,
                ),
            ),
        ),
        [
            UnrecognizedToken {
                token: (
                    1,
                    Token(
                        5,
                        ",",
                    ),
                    2,
                ),
                expected: [
                    "\"(\"",
                    "\")\"",
                    "\"[\"",
                    "\"{\"",
                    "ID_UPPER",
                ],
            },
        ],
    )
    "###);
}

#[test]
fn type_app_zero_args() {
    insta::assert_debug_snapshot!(parse_err("A<>"), @r###"
    (
        Some(
            App(
                Var(
                    t#A,
                ),
                [
                    Error,
                ],
            ),
        ),
        [
            UnrecognizedToken {
                token: (
                    2,
                    Token(
                        17,
                        ">",
                    ),
                    3,
                ),
                expected: [
                    "\"(\"",
                    "\"[\"",
                    "\"{\"",
                    "ID_UPPER",
                ],
            },
        ],
    )
    "###);
}

#[test]
fn record_zero_field_one_comma() {
    insta::assert_debug_snapshot!(parse_err("{,}"), @r###"
    (
        Some(
            Error,
        ),
        [
            UnrecognizedToken {
                token: (
                    1,
                    Token(
                        5,
                        ",",
                    ),
                    2,
                ),
                expected: [
                    "\"}\"",
                    "ID_LOWER",
                ],
            },
        ],
    )
    "###);
}

#[test]
fn variant_zero_constructors() {
    insta::assert_debug_snapshot!(parse_err("[]"), @r###"
    (
        Some(
            Error,
        ),
        [
            UnrecognizedToken {
                token: (
                    1,
                    Token(
                        21,
                        "]",
                    ),
                    2,
                ),
                expected: [
                    "ID_UPPER",
                ],
            },
        ],
    )
    "###);
}
