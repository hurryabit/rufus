use crate::*;
use syntax::*;

use lalrpop_util::ParseError;

fn parse(input: &str) -> Expr {
    let parser = parser::ExprParser::new();
    let mut errors = Vec::new();
    let expr = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    expr
}

fn parse_block(input: &str) -> Expr {
    let parser = parser::BlockExprParser::new();
    let mut errors = Vec::new();
    let expr = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    expr
}

fn parse_err(input: &'static str) -> (Option<Expr>, Vec<ParseError<usize, parser::Token<'static>, &'static str>>) {
    let parser = parser::ExprParser::new();
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
fn var() {
    insta::assert_yaml_snapshot!(parse("x"), @r###"
    ---
    Var: x
    "###);
}

#[test]
fn num() {
    insta::assert_yaml_snapshot!(parse("0"), @r###"
    ---
    Num: 0
    "###);
}

#[test]
fn bool_true() {
    insta::assert_yaml_snapshot!(parse("true"), @r###"
    ---
    Bool: true
    "###);
}

#[test]
fn bool_false() {
    insta::assert_yaml_snapshot!(parse("false"), @r###"
    ---
    Bool: false
    "###);
}

#[test]
fn app0() {
    insta::assert_yaml_snapshot!(parse("f()"), @r###"
    ---
    App:
      - Var: f
      - []
    "###);
}

#[test]
fn app1() {
    insta::assert_yaml_snapshot!(parse("f(1)"), @r###"
    ---
    App:
      - Var: f
      - - Num: 1
    "###);
}

#[test]
fn app1_trailing() {
    insta::assert_yaml_snapshot!(parse("f(1,)"), @r###"
    ---
    App:
      - Var: f
      - - Num: 1
    "###);
}

#[test]
fn app2() {
    insta::assert_yaml_snapshot!(parse("f(1, 2)"), @r###"
    ---
    App:
      - Var: f
      - - Num: 1
        - Num: 2
    "###);
}
#[test]
fn app_ty() {
    insta::assert_yaml_snapshot!(parse("f@<Int>(1)"), @r###"
    ---
    App:
      - TypeApp:
          - Var: f
          - - Var: Int
      - - Num: 1
    "###);
}

#[test]
fn app_ty_err() {
    insta::assert_debug_snapshot!(parse_err("f<A>(1)"), @r###"
    (
        Some(
            BinOp(
                Error,
                Greater,
                Num(
                    1,
                ),
            ),
        ),
        [
            UnrecognizedToken {
                token: (
                    3,
                    Token(
                        17,
                        ">",
                    ),
                    4,
                ),
                expected: [
                    "\")\"",
                    "\"+\"",
                    "\",\"",
                    "\"-\"",
                    "\";\"",
                    "\"{\"",
                    "\"}\"",
                ],
            },
        ],
    )
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
    insta::assert_yaml_snapshot!(parse("{x = 1}"), @r###"
    ---
    Record:
      - - x
        - Num: 1
    "###);
}

#[test]
fn record1_trailing() {
    insta::assert_yaml_snapshot!(parse("{x = 1,}"), @r###"
    ---
    Record:
      - - x
        - Num: 1
    "###);
}

#[test]
fn record2() {
    insta::assert_yaml_snapshot!(parse("{x = 1, y = 2}"), @r###"
    ---
    Record:
      - - x
        - Num: 1
      - - y
        - Num: 2
    "###);
}

#[test]
fn proj1() {
    insta::assert_yaml_snapshot!(parse("r.x"), @r###"
    ---
    Proj:
      - Var: r
      - x
    "###);
}

#[test]
fn proj2() {
    insta::assert_yaml_snapshot!(parse("r.x.y"), @r###"
    ---
    Proj:
      - Proj:
          - Var: r
          - x
      - y
    "###);
}

#[test]
fn variant0() {
    insta::assert_yaml_snapshot!(parse("A"), @r###"
    ---
    Variant:
      - A
      - ~
    "###);
}

#[test]
fn variant1() {
    insta::assert_yaml_snapshot!(parse("A(0)"), @r###"
    ---
    Variant:
      - A
      - Num: 0
    "###);
}

#[test]
fn variant_int() {
    insta::assert_yaml_snapshot!(parse("Int"), @r###"
    ---
    Variant:
      - Int
      - ~
    "###);
}

#[test]
fn variant_bool() {
    insta::assert_yaml_snapshot!(parse("Bool"), @r###"
    ---
    Variant:
      - Bool
      - ~
    "###);
}

#[test]
fn prod2() {
    insta::assert_yaml_snapshot!(parse("a*b"), @r###"
    ---
    BinOp:
      - Var: a
      - Mul
      - Var: b
    "###);
}

#[test]
fn prod3() {
    insta::assert_yaml_snapshot!(parse("a/b*c"), @r###"
    ---
    BinOp:
      - BinOp:
          - Var: a
          - Div
          - Var: b
      - Mul
      - Var: c
    "###);
}

#[test]
fn sum2() {
    insta::assert_yaml_snapshot!(parse("a+b"), @r###"
    ---
    BinOp:
      - Var: a
      - Add
      - Var: b
    "###);
}

#[test]
fn sum3() {
    insta::assert_yaml_snapshot!(parse("a-b+c"), @r###"
    ---
    BinOp:
      - BinOp:
          - Var: a
          - Sub
          - Var: b
      - Add
      - Var: c
    "###);
}

#[test]
fn cmp_eq() {
    insta::assert_yaml_snapshot!(parse("a == b"), @r###"
    ---
    BinOp:
      - Var: a
      - Equals
      - Var: b
    "###);
}

#[test]
fn cmp_neq() {
    insta::assert_yaml_snapshot!(parse("a != b"), @r###"
    ---
    BinOp:
      - Var: a
      - NotEq
      - Var: b
    "###);
}

#[test]
fn cmp_lt() {
    insta::assert_yaml_snapshot!(parse("a < b"), @r###"
    ---
    BinOp:
      - Var: a
      - Less
      - Var: b
    "###);
}

#[test]
fn cmp_leq() {
    insta::assert_yaml_snapshot!(parse("a <= b"), @r###"
    ---
    BinOp:
      - Var: a
      - LessEq
      - Var: b
    "###);
}

#[test]
fn cmp_gt() {
    insta::assert_yaml_snapshot!(parse("a > b"), @r###"
    ---
    BinOp:
      - Var: a
      - Greater
      - Var: b
    "###);
}

#[test]
fn cmp_geq() {
    insta::assert_yaml_snapshot!(parse("a >= b"), @r###"
    ---
    BinOp:
      - Var: a
      - GreaterEq
      - Var: b
    "###);
}

#[test]
fn cmp_mixed() {
    insta::assert_yaml_snapshot!(parse("a + b == c * d"), @r###"
    ---
    BinOp:
      - BinOp:
          - Var: a
          - Add
          - Var: b
      - Equals
      - BinOp:
          - Var: c
          - Mul
          - Var: d
    "###);
}

#[test]
fn cmp_many() {
    insta::assert_yaml_snapshot!(parse("a == (b == c)"), @r###"
    ---
    BinOp:
      - Var: a
      - Equals
      - BinOp:
          - Var: b
          - Equals
          - Var: c
    "###);
}

#[test]
fn cmp_many_err() {
    insta::assert_debug_snapshot!(parse_err("a == b == c"), @r###"
    (
        Some(
            BinOp(
                Error,
                Equals,
                Var(
                    e#c,
                ),
            ),
        ),
        [
            UnrecognizedToken {
                token: (
                    7,
                    Token(
                        15,
                        "==",
                    ),
                    9,
                ),
                expected: [
                    "\")\"",
                    "\"+\"",
                    "\",\"",
                    "\"-\"",
                    "\";\"",
                    "\"{\"",
                    "\"}\"",
                ],
            },
        ],
    )
    "###);
}

#[test]
fn sum_prod() {
    insta::assert_yaml_snapshot!(parse("a+b*c"), @r###"
    ---
    BinOp:
      - Var: a
      - Add
      - BinOp:
          - Var: b
          - Mul
          - Var: c
    "###);
}

#[test]
fn lam0() {
    insta::assert_yaml_snapshot!(parse("fn() { 0 }"), @r###"
    ---
    Lam:
      - []
      - Num: 0
    "###);
}

#[test]
fn lam1() {
    insta::assert_yaml_snapshot!(parse("fn(x) { x }"), @r###"
    ---
    Lam:
      - - - x
          - ~
      - Var: x
    "###);
}

#[test]
fn lam1_trailing() {
    insta::assert_yaml_snapshot!(parse("fn(x,) { x }"), @r###"
    ---
    Lam:
      - - - x
          - ~
      - Var: x
    "###);
}

#[test]
fn lam2() {
    insta::assert_yaml_snapshot!(parse("fn(x, y) { x }"), @r###"
    ---
    Lam:
      - - - x
          - ~
        - - y
          - ~
      - Var: x
    "###);
}
#[test]
fn lam1_typed() {
    insta::assert_yaml_snapshot!(parse("fn(x: Int) { x }"), @r###"
    ---
    Lam:
      - - - x
          - Var: Int
      - Var: x
    "###);
}

#[test]
fn lam1_poly() {
    insta::assert_debug_snapshot!(parse_err("fn<A>(x: A) { x }"), @r###"
    (
        Some(
            Error,
        ),
        [
            UnrecognizedToken {
                token: (
                    2,
                    Token(
                        12,
                        "<",
                    ),
                    3,
                ),
                expected: [
                    "\"(\"",
                ],
            },
            UnrecognizedToken {
                token: (
                    4,
                    Token(
                        17,
                        ">",
                    ),
                    5,
                ),
                expected: [
                    "\")\"",
                    "\"+\"",
                    "\",\"",
                    "\"-\"",
                    "\";\"",
                    "\"{\"",
                    "\"}\"",
                ],
            },
            UnrecognizedToken {
                token: (
                    7,
                    Token(
                        10,
                        ":",
                    ),
                    8,
                ),
                expected: [
                    "\"!=\"",
                    "\"(\"",
                    "\")\"",
                    "\"*\"",
                    "\"+\"",
                    "\",\"",
                    "\"-\"",
                    "\".\"",
                    "\"/\"",
                    "\";\"",
                    "\"<\"",
                    "\"<=\"",
                    "\"==\"",
                    "\">\"",
                    "\">=\"",
                    "\"@\"",
                    "\"{\"",
                    "\"}\"",
                ],
            },
            UnrecognizedToken {
                token: (
                    12,
                    Token(
                        22,
                        "{",
                    ),
                    13,
                ),
                expected: [],
            },
        ],
    )
    "###);
}

#[test]
fn if_atom() {
    insta::assert_yaml_snapshot!(parse("if true { 0 } else { 1 }"), @r###"
    ---
    If:
      - Bool: true
      - Num: 0
      - Num: 1
    "###);
}

#[test]
fn if_cmp() {
    insta::assert_yaml_snapshot!(parse("if a == b { 0 } else { 1 }"), @r###"
    ---
    If:
      - BinOp:
          - Var: a
          - Equals
          - Var: b
      - Num: 0
      - Num: 1
    "###);
}

#[test]
fn block_atom() {
    insta::assert_yaml_snapshot!(parse_block("{ a }"), @r###"
    ---
    Var: a
    "###);
}

#[test]
fn block_record() {
    insta::assert_yaml_snapshot!(parse_block("{ {f = 1} }"), @r###"
    ---
    Record:
      - - f
        - Num: 1
    "###);
}

#[test]
fn let1_atom() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = 1; x }"), @r###"
    ---
    Let:
      - x
      - ~
      - Num: 1
      - Var: x
    "###);
}

#[test]
fn let1_complex() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = 1 + 1; x }"), @r###"
    ---
    Let:
      - x
      - ~
      - BinOp:
          - Num: 1
          - Add
          - Num: 1
      - Var: x
    "###);
}

#[test]
fn let1_typed() {
    insta::assert_yaml_snapshot!(parse_block("{ let x: Int = 1; x }"), @r###"
    ---
    Let:
      - x
      - Var: Int
      - Num: 1
      - Var: x
    "###);
}

#[test]
fn let1_block() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = { 1 }; x }"), @r###"
    ---
    Let:
      - x
      - ~
      - Num: 1
      - Var: x
    "###);
}

#[test]
fn let2() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = 1; let y = x; y }"), @r###"
    ---
    Let:
      - x
      - ~
      - Num: 1
      - Let:
          - y
          - ~
          - Var: x
          - Var: y
    "###);
}

#[test]
fn match1_novar() {
    insta::assert_yaml_snapshot!(parse("match x { A => 1, }"), @r###"
    ---
    Match:
      - Var: x
      - - con: A
          var: ~
          rhs:
            Num: 1
    "###);
}

#[test]
fn match1_var() {
    insta::assert_yaml_snapshot!(parse("match x { A(y) => 1, }"), @r###"
    ---
    Match:
      - Var: x
      - - con: A
          var: y
          rhs:
            Num: 1
    "###);
}

#[test]
fn match1_block() {
    insta::assert_yaml_snapshot!(parse("match x { A => { 1 } }"), @r###"
    ---
    Match:
      - Var: x
      - - con: A
          var: ~
          rhs:
            Num: 1
    "###);
}

#[test]
fn match1_expr_nocomma() {
    insta::assert_debug_snapshot!(parse_err("match x { A => 1 }"), @r###"
    (
        Some(
            Error,
        ),
        [
            UnrecognizedToken {
                token: (
                    17,
                    Token(
                        24,
                        "}",
                    ),
                    18,
                ),
                expected: [
                    "\",\"",
                ],
            },
        ],
    )
    "###);
}

#[test]
fn match1_block_comma() {
    insta::assert_debug_snapshot!(parse_err("match x { A => { 1 }, }"), @r###"
    (
        Some(
            Match(
                Var(
                    e#x,
                ),
                [
                    Branch {
                        con: c#A,
                        var: None,
                        rhs: Error,
                    },
                ],
            ),
        ),
        [
            UnrecognizedToken {
                token: (
                    20,
                    Token(
                        5,
                        ",",
                    ),
                    21,
                ),
                expected: [
                    "\"}\"",
                    "ID_UPPER",
                ],
            },
        ],
    )
    "###);
}

#[test]
fn match2_exprs() {
    insta::assert_yaml_snapshot!(parse("match x { A => 1, B => 2, }"), @r###"
    ---
    Match:
      - Var: x
      - - con: A
          var: ~
          rhs:
            Num: 1
        - con: B
          var: ~
          rhs:
            Num: 2
    "###);
}

#[test]
fn match2_expr_block() {
    insta::assert_yaml_snapshot!(parse("match x { A => 1, B => { 2 } }"), @r###"
    ---
    Match:
      - Var: x
      - - con: A
          var: ~
          rhs:
            Num: 1
        - con: B
          var: ~
          rhs:
            Num: 2
    "###);
}

#[test]
fn match2_block_expr() {
    insta::assert_yaml_snapshot!(parse("match x { A => { 1 } B => 2, }"), @r###"
    ---
    Match:
      - Var: x
      - - con: A
          var: ~
          rhs:
            Num: 1
        - con: B
          var: ~
          rhs:
            Num: 2
    "###);
}

#[test]
fn match2_blocks() {
    insta::assert_yaml_snapshot!(parse("match x { A => { 1 } B => { 2 } }"), @r###"
    ---
    Match:
      - Var: x
      - - con: A
          var: ~
          rhs:
            Num: 1
        - con: B
          var: ~
          rhs:
            Num: 2
    "###);
}
