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

fn parse_err(
    input: &'static str,
) -> (
    Option<Expr>,
    Vec<ParseError<usize, parser::Token<'static>, &'static str>>,
) {
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
      - locatee:
          Var: f
        span:
          start: 0
          end: 1
      - []
    "###);
}

#[test]
fn app1() {
    insta::assert_yaml_snapshot!(parse("f(1)"), @r###"
    ---
    App:
      - locatee:
          Var: f
        span:
          start: 0
          end: 1
      - - locatee:
            Num: 1
          span:
            start: 2
            end: 3
    "###);
}

#[test]
fn app1_trailing() {
    insta::assert_yaml_snapshot!(parse("f(1,)"), @r###"
    ---
    App:
      - locatee:
          Var: f
        span:
          start: 0
          end: 1
      - - locatee:
            Num: 1
          span:
            start: 2
            end: 3
    "###);
}

#[test]
fn app2() {
    insta::assert_yaml_snapshot!(parse("f(1, 2)"), @r###"
    ---
    App:
      - locatee:
          Var: f
        span:
          start: 0
          end: 1
      - - locatee:
            Num: 1
          span:
            start: 2
            end: 3
        - locatee:
            Num: 2
          span:
            start: 5
            end: 6
    "###);
}
#[test]
fn app_ty() {
    insta::assert_yaml_snapshot!(parse("f@<Int>(1)"), @r###"
    ---
    App:
      - locatee:
          FunInst:
            - locatee: f
              span:
                start: 0
                end: 1
            - - locatee:
                  Var: Int
                span:
                  start: 3
                  end: 6
        span:
          start: 0
          end: 7
      - - locatee:
            Num: 1
          span:
            start: 8
            end: 9
    "###);
}

#[test]
fn app_ty_err() {
    insta::assert_debug_snapshot!(parse_err("f<A>(1)"), @r###"
    (
        Some(
            BinOp(
                Located {
                    locatee: Error,
                    span: Span {
                        start: 0,
                        end: 3,
                    },
                },
                Greater,
                Located {
                    locatee: Num(
                        1,
                    ),
                    span: Span {
                        start: 4,
                        end: 7,
                    },
                },
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
      - - locatee: x
          span:
            start: 1
            end: 2
        - locatee:
            Num: 1
          span:
            start: 5
            end: 6
    "###);
}

#[test]
fn record1_trailing() {
    insta::assert_yaml_snapshot!(parse("{x = 1,}"), @r###"
    ---
    Record:
      - - locatee: x
          span:
            start: 1
            end: 2
        - locatee:
            Num: 1
          span:
            start: 5
            end: 6
    "###);
}

#[test]
fn record2() {
    insta::assert_yaml_snapshot!(parse("{x = 1, y = 2}"), @r###"
    ---
    Record:
      - - locatee: x
          span:
            start: 1
            end: 2
        - locatee:
            Num: 1
          span:
            start: 5
            end: 6
      - - locatee: y
          span:
            start: 8
            end: 9
        - locatee:
            Num: 2
          span:
            start: 12
            end: 13
    "###);
}

#[test]
fn proj1() {
    insta::assert_yaml_snapshot!(parse("r.x"), @r###"
    ---
    Proj:
      - locatee:
          Var: r
        span:
          start: 0
          end: 1
      - locatee: x
        span:
          start: 2
          end: 3
    "###);
}

#[test]
fn proj2() {
    insta::assert_yaml_snapshot!(parse("r.x.y"), @r###"
    ---
    Proj:
      - locatee:
          Proj:
            - locatee:
                Var: r
              span:
                start: 0
                end: 1
            - locatee: x
              span:
                start: 2
                end: 3
        span:
          start: 0
          end: 3
      - locatee: y
        span:
          start: 4
          end: 5
    "###);
}

#[test]
fn variant0() {
    insta::assert_yaml_snapshot!(parse("A"), @r###"
    ---
    Variant:
      - locatee: A
        span:
          start: 0
          end: 1
      - locatee:
          Record: []
        span:
          start: 0
          end: 1
    "###);
}

#[test]
fn variant1() {
    insta::assert_yaml_snapshot!(parse("A(0)"), @r###"
    ---
    Variant:
      - locatee: A
        span:
          start: 0
          end: 1
      - locatee:
          Num: 0
        span:
          start: 2
          end: 3
    "###);
}

#[test]
fn variant_int() {
    insta::assert_yaml_snapshot!(parse("Int"), @r###"
    ---
    Variant:
      - locatee: Int
        span:
          start: 0
          end: 3
      - locatee:
          Record: []
        span:
          start: 0
          end: 3
    "###);
}

#[test]
fn variant_bool() {
    insta::assert_yaml_snapshot!(parse("Bool"), @r###"
    ---
    Variant:
      - locatee: Bool
        span:
          start: 0
          end: 4
      - locatee:
          Record: []
        span:
          start: 0
          end: 4
    "###);
}

#[test]
fn prod2() {
    insta::assert_yaml_snapshot!(parse("a*b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - Mul
      - locatee:
          Var: b
        span:
          start: 2
          end: 3
    "###);
}

#[test]
fn prod3() {
    insta::assert_yaml_snapshot!(parse("a/b*c"), @r###"
    ---
    BinOp:
      - locatee:
          BinOp:
            - locatee:
                Var: a
              span:
                start: 0
                end: 1
            - Div
            - locatee:
                Var: b
              span:
                start: 2
                end: 3
        span:
          start: 0
          end: 3
      - Mul
      - locatee:
          Var: c
        span:
          start: 4
          end: 5
    "###);
}

#[test]
fn sum2() {
    insta::assert_yaml_snapshot!(parse("a+b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - Add
      - locatee:
          Var: b
        span:
          start: 2
          end: 3
    "###);
}

#[test]
fn sum3() {
    insta::assert_yaml_snapshot!(parse("a-b+c"), @r###"
    ---
    BinOp:
      - locatee:
          BinOp:
            - locatee:
                Var: a
              span:
                start: 0
                end: 1
            - Sub
            - locatee:
                Var: b
              span:
                start: 2
                end: 3
        span:
          start: 0
          end: 3
      - Add
      - locatee:
          Var: c
        span:
          start: 4
          end: 5
    "###);
}

#[test]
fn cmp_eq() {
    insta::assert_yaml_snapshot!(parse("a == b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - Equals
      - locatee:
          Var: b
        span:
          start: 5
          end: 6
    "###);
}

#[test]
fn cmp_neq() {
    insta::assert_yaml_snapshot!(parse("a != b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - NotEq
      - locatee:
          Var: b
        span:
          start: 5
          end: 6
    "###);
}

#[test]
fn cmp_lt() {
    insta::assert_yaml_snapshot!(parse("a < b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - Less
      - locatee:
          Var: b
        span:
          start: 4
          end: 5
    "###);
}

#[test]
fn cmp_leq() {
    insta::assert_yaml_snapshot!(parse("a <= b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - LessEq
      - locatee:
          Var: b
        span:
          start: 5
          end: 6
    "###);
}

#[test]
fn cmp_gt() {
    insta::assert_yaml_snapshot!(parse("a > b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - Greater
      - locatee:
          Var: b
        span:
          start: 4
          end: 5
    "###);
}

#[test]
fn cmp_geq() {
    insta::assert_yaml_snapshot!(parse("a >= b"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - GreaterEq
      - locatee:
          Var: b
        span:
          start: 5
          end: 6
    "###);
}

#[test]
fn cmp_mixed() {
    insta::assert_yaml_snapshot!(parse("a + b == c * d"), @r###"
    ---
    BinOp:
      - locatee:
          BinOp:
            - locatee:
                Var: a
              span:
                start: 0
                end: 1
            - Add
            - locatee:
                Var: b
              span:
                start: 4
                end: 5
        span:
          start: 0
          end: 5
      - Equals
      - locatee:
          BinOp:
            - locatee:
                Var: c
              span:
                start: 9
                end: 10
            - Mul
            - locatee:
                Var: d
              span:
                start: 13
                end: 14
        span:
          start: 9
          end: 14
    "###);
}

#[test]
fn cmp_many() {
    insta::assert_yaml_snapshot!(parse("a == (b == c)"), @r###"
    ---
    BinOp:
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - Equals
      - locatee:
          BinOp:
            - locatee:
                Var: b
              span:
                start: 6
                end: 7
            - Equals
            - locatee:
                Var: c
              span:
                start: 11
                end: 12
        span:
          start: 5
          end: 13
    "###);
}

#[test]
fn cmp_many_err() {
    insta::assert_debug_snapshot!(parse_err("a == b == c"), @r###"
    (
        Some(
            BinOp(
                Located {
                    locatee: Error,
                    span: Span {
                        start: 0,
                        end: 6,
                    },
                },
                Equals,
                Located {
                    locatee: Var(
                        e#c,
                    ),
                    span: Span {
                        start: 10,
                        end: 11,
                    },
                },
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
      - locatee:
          Var: a
        span:
          start: 0
          end: 1
      - Add
      - locatee:
          BinOp:
            - locatee:
                Var: b
              span:
                start: 2
                end: 3
            - Mul
            - locatee:
                Var: c
              span:
                start: 4
                end: 5
        span:
          start: 2
          end: 5
    "###);
}

#[test]
fn lam0() {
    insta::assert_yaml_snapshot!(parse("fn() { 0 }"), @r###"
    ---
    Lam:
      - []
      - locatee:
          Num: 0
        span:
          start: 5
          end: 10
    "###);
}

#[test]
fn lam1() {
    insta::assert_yaml_snapshot!(parse("fn(x) { x }"), @r###"
    ---
    Lam:
      - - - locatee: x
            span:
              start: 3
              end: 4
          - ~
      - locatee:
          Var: x
        span:
          start: 6
          end: 11
    "###);
}

#[test]
fn lam1_trailing() {
    insta::assert_yaml_snapshot!(parse("fn(x,) { x }"), @r###"
    ---
    Lam:
      - - - locatee: x
            span:
              start: 3
              end: 4
          - ~
      - locatee:
          Var: x
        span:
          start: 7
          end: 12
    "###);
}

#[test]
fn lam2() {
    insta::assert_yaml_snapshot!(parse("fn(x, y) { x }"), @r###"
    ---
    Lam:
      - - - locatee: x
            span:
              start: 3
              end: 4
          - ~
        - - locatee: y
            span:
              start: 6
              end: 7
          - ~
      - locatee:
          Var: x
        span:
          start: 9
          end: 14
    "###);
}
#[test]
fn lam1_typed() {
    insta::assert_yaml_snapshot!(parse("fn(x: Int) { x }"), @r###"
    ---
    Lam:
      - - - locatee: x
            span:
              start: 3
              end: 4
          - locatee:
              Var: Int
            span:
              start: 6
              end: 9
      - locatee:
          Var: x
        span:
          start: 11
          end: 16
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
                    "\"=\"",
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
      - locatee:
          Bool: true
        span:
          start: 3
          end: 7
      - locatee:
          Num: 0
        span:
          start: 8
          end: 13
      - locatee:
          Num: 1
        span:
          start: 19
          end: 24
    "###);
}

#[test]
fn if_cmp() {
    insta::assert_yaml_snapshot!(parse("if a == b { 0 } else { 1 }"), @r###"
    ---
    If:
      - locatee:
          BinOp:
            - locatee:
                Var: a
              span:
                start: 3
                end: 4
            - Equals
            - locatee:
                Var: b
              span:
                start: 8
                end: 9
        span:
          start: 3
          end: 9
      - locatee:
          Num: 0
        span:
          start: 10
          end: 15
      - locatee:
          Num: 1
        span:
          start: 21
          end: 26
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
      - - locatee: f
          span:
            start: 3
            end: 4
        - locatee:
            Num: 1
          span:
            start: 7
            end: 8
    "###);
}

#[test]
fn let1_atom() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = 1; x }"), @r###"
    ---
    Let:
      - locatee: x
        span:
          start: 6
          end: 7
      - ~
      - locatee:
          Num: 1
        span:
          start: 10
          end: 11
      - locatee:
          Var: x
        span:
          start: 13
          end: 14
    "###);
}

#[test]
fn let1_complex() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = 1 + 1; x }"), @r###"
    ---
    Let:
      - locatee: x
        span:
          start: 6
          end: 7
      - ~
      - locatee:
          BinOp:
            - locatee:
                Num: 1
              span:
                start: 10
                end: 11
            - Add
            - locatee:
                Num: 1
              span:
                start: 14
                end: 15
        span:
          start: 10
          end: 15
      - locatee:
          Var: x
        span:
          start: 17
          end: 18
    "###);
}

#[test]
fn let1_typed() {
    insta::assert_yaml_snapshot!(parse_block("{ let x: Int = 1; x }"), @r###"
    ---
    Let:
      - locatee: x
        span:
          start: 6
          end: 7
      - locatee:
          Var: Int
        span:
          start: 9
          end: 12
      - locatee:
          Num: 1
        span:
          start: 15
          end: 16
      - locatee:
          Var: x
        span:
          start: 18
          end: 19
    "###);
}

#[test]
fn let1_block() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = { 1 }; x }"), @r###"
    ---
    Let:
      - locatee: x
        span:
          start: 6
          end: 7
      - ~
      - locatee:
          Num: 1
        span:
          start: 10
          end: 15
      - locatee:
          Var: x
        span:
          start: 17
          end: 18
    "###);
}

#[test]
fn let2() {
    insta::assert_yaml_snapshot!(parse_block("{ let x = 1; let y = x; y }"), @r###"
    ---
    Let:
      - locatee: x
        span:
          start: 6
          end: 7
      - ~
      - locatee:
          Num: 1
        span:
          start: 10
          end: 11
      - locatee:
          Let:
            - locatee: y
              span:
                start: 17
                end: 18
            - ~
            - locatee:
                Var: x
              span:
                start: 21
                end: 22
            - locatee:
                Var: y
              span:
                start: 24
                end: 25
        span:
          start: 13
          end: 25
    "###);
}

#[test]
fn match1_novar() {
    insta::assert_yaml_snapshot!(parse("match x { A => 1, }"), @r###"
    ---
    Match:
      - locatee:
          Var: x
        span:
          start: 6
          end: 7
      - - locatee:
            con:
              locatee: A
              span:
                start: 10
                end: 11
            var: ~
            rhs:
              locatee:
                Num: 1
              span:
                start: 15
                end: 16
          span:
            start: 10
            end: 17
    "###);
}

#[test]
fn match1_var() {
    insta::assert_yaml_snapshot!(parse("match x { A(y) => 1, }"), @r###"
    ---
    Match:
      - locatee:
          Var: x
        span:
          start: 6
          end: 7
      - - locatee:
            con:
              locatee: A
              span:
                start: 10
                end: 11
            var:
              locatee: y
              span:
                start: 12
                end: 13
            rhs:
              locatee:
                Num: 1
              span:
                start: 18
                end: 19
          span:
            start: 10
            end: 20
    "###);
}

#[test]
fn match1_block() {
    insta::assert_yaml_snapshot!(parse("match x { A => { 1 } }"), @r###"
    ---
    Match:
      - locatee:
          Var: x
        span:
          start: 6
          end: 7
      - - locatee:
            con:
              locatee: A
              span:
                start: 10
                end: 11
            var: ~
            rhs:
              locatee:
                Num: 1
              span:
                start: 15
                end: 20
          span:
            start: 10
            end: 20
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
                Located {
                    locatee: Var(
                        e#x,
                    ),
                    span: Span {
                        start: 6,
                        end: 7,
                    },
                },
                [
                    Located {
                        locatee: Branch {
                            con: Located {
                                locatee: c#A,
                                span: Span {
                                    start: 10,
                                    end: 11,
                                },
                            },
                            var: None,
                            rhs: Located {
                                locatee: Error,
                                span: Span {
                                    start: 15,
                                    end: 20,
                                },
                            },
                        },
                        span: Span {
                            start: 10,
                            end: 21,
                        },
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
      - locatee:
          Var: x
        span:
          start: 6
          end: 7
      - - locatee:
            con:
              locatee: A
              span:
                start: 10
                end: 11
            var: ~
            rhs:
              locatee:
                Num: 1
              span:
                start: 15
                end: 16
          span:
            start: 10
            end: 17
        - locatee:
            con:
              locatee: B
              span:
                start: 18
                end: 19
            var: ~
            rhs:
              locatee:
                Num: 2
              span:
                start: 23
                end: 24
          span:
            start: 18
            end: 25
    "###);
}

#[test]
fn match2_expr_block() {
    insta::assert_yaml_snapshot!(parse("match x { A => 1, B => { 2 } }"), @r###"
    ---
    Match:
      - locatee:
          Var: x
        span:
          start: 6
          end: 7
      - - locatee:
            con:
              locatee: A
              span:
                start: 10
                end: 11
            var: ~
            rhs:
              locatee:
                Num: 1
              span:
                start: 15
                end: 16
          span:
            start: 10
            end: 17
        - locatee:
            con:
              locatee: B
              span:
                start: 18
                end: 19
            var: ~
            rhs:
              locatee:
                Num: 2
              span:
                start: 23
                end: 28
          span:
            start: 18
            end: 28
    "###);
}

#[test]
fn match2_block_expr() {
    insta::assert_yaml_snapshot!(parse("match x { A => { 1 } B => 2, }"), @r###"
    ---
    Match:
      - locatee:
          Var: x
        span:
          start: 6
          end: 7
      - - locatee:
            con:
              locatee: A
              span:
                start: 10
                end: 11
            var: ~
            rhs:
              locatee:
                Num: 1
              span:
                start: 15
                end: 20
          span:
            start: 10
            end: 20
        - locatee:
            con:
              locatee: B
              span:
                start: 21
                end: 22
            var: ~
            rhs:
              locatee:
                Num: 2
              span:
                start: 26
                end: 27
          span:
            start: 21
            end: 28
    "###);
}

#[test]
fn match2_blocks() {
    insta::assert_yaml_snapshot!(parse("match x { A => { 1 } B => { 2 } }"), @r###"
    ---
    Match:
      - locatee:
          Var: x
        span:
          start: 6
          end: 7
      - - locatee:
            con:
              locatee: A
              span:
                start: 10
                end: 11
            var: ~
            rhs:
              locatee:
                Num: 1
              span:
                start: 15
                end: 20
          span:
            start: 10
            end: 20
        - locatee:
            con:
              locatee: B
              span:
                start: 21
                end: 22
            var: ~
            rhs:
              locatee:
                Num: 2
              span:
                start: 26
                end: 31
          span:
            start: 21
            end: 31
    "###);
}
