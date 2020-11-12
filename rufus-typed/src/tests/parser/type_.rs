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
    Var:
      locatee: A
      span:
        start: 0
        end: 1
    "###);
}

#[test]
fn func0() {
    insta::assert_yaml_snapshot!(parse("() -> Int"), @r###"
    ---
    Fun:
      - []
      - locatee:
          Var:
            locatee: Int
            span:
              start: 6
              end: 9
        span:
          start: 6
          end: 9
    "###);
}

#[test]
fn func1() {
    insta::assert_yaml_snapshot!(parse("(Int) -> Int"), @r###"
    ---
    Fun:
      - - locatee:
            Var:
              locatee: Int
              span:
                start: 1
                end: 4
          span:
            start: 1
            end: 4
      - locatee:
          Var:
            locatee: Int
            span:
              start: 9
              end: 12
        span:
          start: 9
          end: 12
    "###);
}

#[test]
fn func1_extra_comma() {
    insta::assert_yaml_snapshot!(parse("(Int,) -> Int"), @r###"
    ---
    Fun:
      - - locatee:
            Var:
              locatee: Int
              span:
                start: 1
                end: 4
          span:
            start: 1
            end: 4
      - locatee:
          Var:
            locatee: Int
            span:
              start: 10
              end: 13
        span:
          start: 10
          end: 13
    "###);
}

#[test]
fn syn_app1() {
    insta::assert_yaml_snapshot!(parse("A<Int>"), @r###"
    ---
    SynApp:
      - locatee: A
        span:
          start: 0
          end: 1
      - - locatee:
            Var:
              locatee: Int
              span:
                start: 2
                end: 5
          span:
            start: 2
            end: 5
    "###);
}

#[test]
fn syn_app1_extra_comma() {
    insta::assert_yaml_snapshot!(parse("A<Int,>"), @r###"
    ---
    SynApp:
      - locatee: A
        span:
          start: 0
          end: 1
      - - locatee:
            Var:
              locatee: Int
              span:
                start: 2
                end: 5
          span:
            start: 2
            end: 5
    "###);
}

#[test]
fn syn_app2() {
    insta::assert_yaml_snapshot!(parse("A<Int, Bool>"), @r###"
    ---
    SynApp:
      - locatee: A
        span:
          start: 0
          end: 1
      - - locatee:
            Var:
              locatee: Int
              span:
                start: 2
                end: 5
          span:
            start: 2
            end: 5
        - locatee:
            Var:
              locatee: Bool
              span:
                start: 7
                end: 11
          span:
            start: 7
            end: 11
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
      - - locatee: x
          span:
            start: 1
            end: 2
        - locatee:
            Var:
              locatee: Int
              span:
                start: 4
                end: 7
          span:
            start: 4
            end: 7
    "###);
}

#[test]
fn record1_extra_comma() {
    insta::assert_yaml_snapshot!(parse("{x: Int,}"), @r###"
    ---
    Record:
      - - locatee: x
          span:
            start: 1
            end: 2
        - locatee:
            Var:
              locatee: Int
              span:
                start: 4
                end: 7
          span:
            start: 4
            end: 7
    "###);
}

#[test]
fn variant1_unit() {
    insta::assert_yaml_snapshot!(parse("[A]"), @r###"
    ---
    Variant:
      - - locatee: A
          span:
            start: 1
            end: 2
        - locatee:
            Record: []
          span:
            start: 1
            end: 2
    "###);
}

#[test]
fn variant1_payload() {
    insta::assert_yaml_snapshot!(parse("[A(Int)]"), @r###"
    ---
    Variant:
      - - locatee: A
          span:
            start: 1
            end: 2
        - locatee:
            Var:
              locatee: Int
              span:
                start: 3
                end: 6
          span:
            start: 3
            end: 6
    "###);
}

#[test]
fn variant2_units() {
    insta::assert_yaml_snapshot!(parse("[A | B]"), @r###"
    ---
    Variant:
      - - locatee: A
          span:
            start: 1
            end: 2
        - locatee:
            Record: []
          span:
            start: 1
            end: 2
      - - locatee: B
          span:
            start: 5
            end: 6
        - locatee:
            Record: []
          span:
            start: 5
            end: 6
    "###);
}

#[test]
fn variant2_unit_payload() {
    insta::assert_yaml_snapshot!(parse("[A | B(Int)]"), @r###"
    ---
    Variant:
      - - locatee: A
          span:
            start: 1
            end: 2
        - locatee:
            Record: []
          span:
            start: 1
            end: 2
      - - locatee: B
          span:
            start: 5
            end: 6
        - locatee:
            Var:
              locatee: Int
              span:
                start: 7
                end: 10
          span:
            start: 7
            end: 10
    "###);
}

#[test]
fn variant2_payload_unit() {
    insta::assert_yaml_snapshot!(parse("[A(Bool) | B]"), @r###"
    ---
    Variant:
      - - locatee: A
          span:
            start: 1
            end: 2
        - locatee:
            Var:
              locatee: Bool
              span:
                start: 3
                end: 7
          span:
            start: 3
            end: 7
      - - locatee: B
          span:
            start: 11
            end: 12
        - locatee:
            Record: []
          span:
            start: 11
            end: 12
    "###);
}

#[test]
fn variant2_payloads() {
    insta::assert_yaml_snapshot!(parse("[A(Bool) | B(Int)]"), @r###"
    ---
    Variant:
      - - locatee: A
          span:
            start: 1
            end: 2
        - locatee:
            Var:
              locatee: Bool
              span:
                start: 3
                end: 7
          span:
            start: 3
            end: 7
      - - locatee: B
          span:
            start: 11
            end: 12
        - locatee:
            Var:
              locatee: Int
              span:
                start: 13
                end: 16
          span:
            start: 13
            end: 16
    "###);
}

// TODO(MH): We want to allow an optional leading "|" rather
// than a trailing one.
#[test]
fn variant2_extra_bar() {
    insta::assert_yaml_snapshot!(parse("[A | B(Int) |]"), @r###"
    ---
    Variant:
      - - locatee: A
          span:
            start: 1
            end: 2
        - locatee:
            Record: []
          span:
            start: 1
            end: 2
      - - locatee: B
          span:
            start: 5
            end: 6
        - locatee:
            Var:
              locatee: Int
              span:
                start: 7
                end: 10
          span:
            start: 7
            end: 10
    "###);
}

#[test]
fn func_type_zero_params_one_comma() {
    insta::assert_debug_snapshot!(parse_err("(,) -> Int"), @r###"
    (
        Some(
            Fun(
                [
                    Located {
                        locatee: Error,
                        span: Span {
                            start: 1,
                            end: 1,
                        },
                    },
                ],
                Located {
                    locatee: Var(
                        Located {
                            locatee: t#Int,
                            span: Span {
                                start: 7,
                                end: 10,
                            },
                        },
                    ),
                    span: Span {
                        start: 7,
                        end: 10,
                    },
                },
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
                Located {
                    locatee: t#A,
                    span: Span {
                        start: 0,
                        end: 1,
                    },
                },
                [
                    Located {
                        locatee: Error,
                        span: Span {
                            start: 2,
                            end: 2,
                        },
                    },
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
