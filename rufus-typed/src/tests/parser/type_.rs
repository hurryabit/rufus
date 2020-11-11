use crate::*;
use syntax::*;

use lalrpop_util::ParseError;

fn parse(input: &'static str) -> Type {
    let parser = parser::TypeParser::new();
    let mut errors = Vec::new();
    let typ = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    typ
}

fn parse_err(
    input: &'static str,
) -> (
    Option<Type>,
    Vec<ParseError<usize, parser::Token<'static>, &'static str>>,
) {
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

#[test]
fn type_var() {
    insta::assert_yaml_snapshot!(parse("A"), @r###"
    ---
    Var: A
    "###);
}

#[test]
fn func0() {
    insta::assert_yaml_snapshot!(parse("() -> Int"), @r###"
    ---
    Fun:
      - []
      - Var: Int
    "###);
}

#[test]
fn func1() {
    insta::assert_yaml_snapshot!(parse("(Int) -> Int"), @r###"
    ---
    Fun:
      - - Var: Int
      - Var: Int
    "###);
}

#[test]
fn func1_extra_comma() {
    insta::assert_yaml_snapshot!(parse("(Int,) -> Int"), @r###"
    ---
    Fun:
      - - Var: Int
      - Var: Int
    "###);
}

#[test]
fn syn_app1() {
    insta::assert_yaml_snapshot!(parse("A<Int>"), @r###"
    ---
    SynApp:
      - A
      - - Var: Int
    "###);
}

#[test]
fn syn_app1_extra_comma() {
    insta::assert_yaml_snapshot!(parse("A<Int,>"), @r###"
    ---
    SynApp:
      - A
      - - Var: Int
    "###);
}

#[test]
fn syn_app2() {
    insta::assert_yaml_snapshot!(parse("A<Int, Bool>"), @r###"
    ---
    SynApp:
      - A
      - - Var: Int
        - Var: Bool
    "###);
}

#[test]
fn record0() {
    insta::assert_yaml_snapshot!(parse("{}"), @r###"
    ---
    Record: []
    "###);
}

#[test]
fn record1() {
    insta::assert_yaml_snapshot!(parse("{x: Int}"), @r###"
    ---
    Record:
      - - x
        - Var: Int
    "###);
}

#[test]
fn record1_extra_comma() {
    insta::assert_yaml_snapshot!(parse("{x: Int,}"), @r###"
    ---
    Record:
      - - x
        - Var: Int
    "###);
}

#[test]
fn variant1_unit() {
    insta::assert_yaml_snapshot!(parse("[A]"), @r###"
    ---
    Variant:
      - - A
        - Record: []
    "###);
}

#[test]
fn variant1_payload() {
    insta::assert_yaml_snapshot!(parse("[A(Int)]"), @r###"
    ---
    Variant:
      - - A
        - Var: Int
    "###);
}

#[test]
fn variant2_units() {
    insta::assert_yaml_snapshot!(parse("[A | B]"), @r###"
    ---
    Variant:
      - - A
        - Record: []
      - - B
        - Record: []
    "###);
}

#[test]
fn variant2_unit_payload() {
    insta::assert_yaml_snapshot!(parse("[A | B(Int)]"), @r###"
    ---
    Variant:
      - - A
        - Record: []
      - - B
        - Var: Int
    "###);
}

#[test]
fn variant2_payload_unit() {
    insta::assert_yaml_snapshot!(parse("[A(Bool) | B]"), @r###"
    ---
    Variant:
      - - A
        - Var: Bool
      - - B
        - Record: []
    "###);
}

#[test]
fn variant2_payloads() {
    insta::assert_yaml_snapshot!(parse("[A(Bool) | B(Int)]"), @r###"
    ---
    Variant:
      - - A
        - Var: Bool
      - - B
        - Var: Int
    "###);
}

// TODO(MH): We want to allow an optional leading "|" rather
// than a trailing one.
#[test]
fn variant2_extra_bar() {
    insta::assert_yaml_snapshot!(parse("[A | B(Int) |]"), @r###"
    ---
    Variant:
      - - A
        - Record: []
      - - B
        - Var: Int
    "###);
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
            SynApp(
                t#A,
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
