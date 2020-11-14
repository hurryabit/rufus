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
    let parser = parser::LBlockExprParser::new();
    let mut errors = Vec::new();
    let expr = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    expr.locatee
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
    insta::assert_debug_snapshot!(parse("x"), @"x");
}

#[test]
fn num() {
    insta::assert_debug_snapshot!(parse("0"), @"0");
}

#[test]
fn bool_true() {
    insta::assert_debug_snapshot!(parse("true"), @"true");
}

#[test]
fn bool_false() {
    insta::assert_debug_snapshot!(parse("false"), @"false");
}

#[test]
fn app0() {
    insta::assert_debug_snapshot!(parse("f()"), @r###"
    APP
      fun: f @ 0...1
    "###);
}

#[test]
fn app1() {
    insta::assert_debug_snapshot!(parse("f(1)"), @r###"
    APP
      fun: f @ 0...1
      arg: 1 @ 2...3
    "###);
}

#[test]
fn app1_trailing() {
    insta::assert_debug_snapshot!(parse("f(1,)"), @r###"
    APP
      fun: f @ 0...1
      arg: 1 @ 2...3
    "###);
}

#[test]
fn app2() {
    insta::assert_debug_snapshot!(parse("f(1, 2)"), @r###"
    APP
      fun: f @ 0...1
      arg: 1 @ 2...3
      arg: 2 @ 5...6
    "###);
}
#[test]
fn app_ty() {
    insta::assert_debug_snapshot!(parse("f@<Int>(1)"), @r###"
    APP
      fun: FUNINST @ 0...7
        fun: f @ 0...1
        type_arg: Int @ 3...6
      arg: 1 @ 8...9
    "###);
}

#[test]
fn app_ty_err() {
    insta::assert_debug_snapshot!(parse_err("f<A>(1)"), @r###"
    (
        Some(
            BINOP
              lhs: ERROR @ 0...3
              op: GREATER
              rhs: 1 @ 4...7,
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
    insta::assert_debug_snapshot!(parse("{}"), @"RECORD");
}
#[test]
fn record1() {
    insta::assert_debug_snapshot!(parse("{x = 1}"), @r###"
    RECORD
      field: x @ 1...2
      value: 1 @ 5...6
    "###);
}

#[test]
fn record1_trailing() {
    insta::assert_debug_snapshot!(parse("{x = 1,}"), @r###"
    RECORD
      field: x @ 1...2
      value: 1 @ 5...6
    "###);
}

#[test]
fn record2() {
    insta::assert_debug_snapshot!(parse("{x = 1, y = 2}"), @r###"
    RECORD
      field: x @ 1...2
      value: 1 @ 5...6
      field: y @ 8...9
      value: 2 @ 12...13
    "###);
}

#[test]
fn proj1() {
    insta::assert_debug_snapshot!(parse("r.x"), @r###"
    PROJ
      record: r @ 0...1
      field: x @ 2...3
    "###);
}

#[test]
fn proj2() {
    insta::assert_debug_snapshot!(parse("r.x.y"), @r###"
    PROJ
      record: PROJ @ 0...3
        record: r @ 0...1
        field: x @ 2...3
      field: y @ 4...5
    "###);
}

#[test]
fn variant0() {
    insta::assert_debug_snapshot!(parse("A"), @r###"
    VARIANT
      constr: A @ 0...1
      payload: RECORD @ 0...1
    "###);
}

#[test]
fn variant1() {
    insta::assert_debug_snapshot!(parse("A(0)"), @r###"
    VARIANT
      constr: A @ 0...1
      payload: 0 @ 2...3
    "###);
}

#[test]
fn variant_int() {
    insta::assert_debug_snapshot!(parse("Int"), @r###"
    VARIANT
      constr: Int @ 0...3
      payload: RECORD @ 0...3
    "###);
}

#[test]
fn variant_bool() {
    insta::assert_debug_snapshot!(parse("Bool"), @r###"
    VARIANT
      constr: Bool @ 0...4
      payload: RECORD @ 0...4
    "###);
}

#[test]
fn prod2() {
    insta::assert_debug_snapshot!(parse("a*b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: MUL
      rhs: b @ 2...3
    "###);
}

#[test]
fn prod3() {
    insta::assert_debug_snapshot!(parse("a/b*c"), @r###"
    BINOP
      lhs: BINOP @ 0...3
        lhs: a @ 0...1
        op: DIV
        rhs: b @ 2...3
      op: MUL
      rhs: c @ 4...5
    "###);
}

#[test]
fn sum2() {
    insta::assert_debug_snapshot!(parse("a+b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: ADD
      rhs: b @ 2...3
    "###);
}

#[test]
fn sum3() {
    insta::assert_debug_snapshot!(parse("a-b+c"), @r###"
    BINOP
      lhs: BINOP @ 0...3
        lhs: a @ 0...1
        op: SUB
        rhs: b @ 2...3
      op: ADD
      rhs: c @ 4...5
    "###);
}

#[test]
fn cmp_eq() {
    insta::assert_debug_snapshot!(parse("a == b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: EQUALS
      rhs: b @ 5...6
    "###);
}

#[test]
fn cmp_neq() {
    insta::assert_debug_snapshot!(parse("a != b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: NOTEQ
      rhs: b @ 5...6
    "###);
}

#[test]
fn cmp_lt() {
    insta::assert_debug_snapshot!(parse("a < b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: LESS
      rhs: b @ 4...5
    "###);
}

#[test]
fn cmp_leq() {
    insta::assert_debug_snapshot!(parse("a <= b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: LESSEQ
      rhs: b @ 5...6
    "###);
}

#[test]
fn cmp_gt() {
    insta::assert_debug_snapshot!(parse("a > b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: GREATER
      rhs: b @ 4...5
    "###);
}

#[test]
fn cmp_geq() {
    insta::assert_debug_snapshot!(parse("a >= b"), @r###"
    BINOP
      lhs: a @ 0...1
      op: GREATEREQ
      rhs: b @ 5...6
    "###);
}

#[test]
fn cmp_mixed() {
    insta::assert_debug_snapshot!(parse("a + b == c * d"), @r###"
    BINOP
      lhs: BINOP @ 0...5
        lhs: a @ 0...1
        op: ADD
        rhs: b @ 4...5
      op: EQUALS
      rhs: BINOP @ 9...14
        lhs: c @ 9...10
        op: MUL
        rhs: d @ 13...14
    "###);
}

#[test]
fn cmp_many() {
    insta::assert_debug_snapshot!(parse("a == (b == c)"), @r###"
    BINOP
      lhs: a @ 0...1
      op: EQUALS
      rhs: BINOP @ 5...13
        lhs: b @ 6...7
        op: EQUALS
        rhs: c @ 11...12
    "###);
}

#[test]
fn cmp_many_err() {
    insta::assert_debug_snapshot!(parse_err("a == b == c"), @r###"
    (
        Some(
            BINOP
              lhs: ERROR @ 0...6
              op: EQUALS
              rhs: c @ 10...11,
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
    insta::assert_debug_snapshot!(parse("a+b*c"), @r###"
    BINOP
      lhs: a @ 0...1
      op: ADD
      rhs: BINOP @ 2...5
        lhs: b @ 2...3
        op: MUL
        rhs: c @ 4...5
    "###);
}

#[test]
fn lam0() {
    insta::assert_debug_snapshot!(parse("fn() { 0 }"), @r###"
    LAM
      body: 0 @ 7...8
    "###);
}

#[test]
fn lam1() {
    insta::assert_debug_snapshot!(parse("fn(x) { x }"), @r###"
    LAM
      param: x @ 3...4
      body: x @ 8...9
    "###);
}

#[test]
fn lam1_trailing() {
    insta::assert_debug_snapshot!(parse("fn(x,) { x }"), @r###"
    LAM
      param: x @ 3...4
      body: x @ 9...10
    "###);
}

#[test]
fn lam2() {
    insta::assert_debug_snapshot!(parse("fn(x, y) { x }"), @r###"
    LAM
      param: x @ 3...4
      param: y @ 6...7
      body: x @ 11...12
    "###);
}
#[test]
fn lam1_typed() {
    insta::assert_debug_snapshot!(parse("fn(x: Int) { x }"), @r###"
    LAM
      param: x @ 3...4
      type: Int @ 6...9
      body: x @ 13...14
    "###);
}

#[test]
fn lam1_poly() {
    insta::assert_debug_snapshot!(parse_err("fn<A>(x: A) { x }"), @r###"
    (
        Some(
            ERROR,
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
    insta::assert_debug_snapshot!(parse("if true { 0 } else { 1 }"), @r###"
    IF
      cond: true @ 3...7
      then: 0 @ 10...11
      else: 1 @ 21...22
    "###);
}

#[test]
fn if_cmp() {
    insta::assert_debug_snapshot!(parse("if a == b { 0 } else { 1 }"), @r###"
    IF
      cond: BINOP @ 3...9
        lhs: a @ 3...4
        op: EQUALS
        rhs: b @ 8...9
      then: 0 @ 12...13
      else: 1 @ 23...24
    "###);
}

#[test]
fn block_atom() {
    insta::assert_debug_snapshot!(parse_block("{ a }"), @"a");
}

#[test]
fn block_record() {
    insta::assert_debug_snapshot!(parse_block("{ {f = 1} }"), @r###"
    RECORD
      field: f @ 3...4
      value: 1 @ 7...8
    "###);
}

#[test]
fn let1_atom() {
    insta::assert_debug_snapshot!(parse_block("{ let x = 1; x }"), @r###"
    LET
      binder: x @ 6...7
      bindee: 1 @ 10...11
      body: x @ 13...14
    "###);
}

#[test]
fn let1_complex() {
    insta::assert_debug_snapshot!(parse_block("{ let x = 1 + 1; x }"), @r###"
    LET
      binder: x @ 6...7
      bindee: BINOP @ 10...15
        lhs: 1 @ 10...11
        op: ADD
        rhs: 1 @ 14...15
      body: x @ 17...18
    "###);
}

#[test]
fn let1_typed() {
    insta::assert_debug_snapshot!(parse_block("{ let x: Int = 1; x }"), @r###"
    LET
      binder: x @ 6...7
      type: Int @ 9...12
      bindee: 1 @ 15...16
      body: x @ 18...19
    "###);
}

#[test]
fn let1_block() {
    insta::assert_debug_snapshot!(parse_block("{ let x = { 1 }; x }"), @r###"
    LET
      binder: x @ 6...7
      bindee: 1 @ 12...13
      body: x @ 17...18
    "###);
}

#[test]
fn let2() {
    insta::assert_debug_snapshot!(parse_block("{ let x = 1; let y = x; y }"), @r###"
    LET
      binder: x @ 6...7
      bindee: 1 @ 10...11
      body: LET @ 13...25
        binder: y @ 17...18
        bindee: x @ 21...22
        body: y @ 24...25
    "###);
}

#[test]
fn match1_novar() {
    insta::assert_debug_snapshot!(parse("match x { A => 1, }"), @r###"
    MATCH
      scrut: x @ 6...7
      branch: BRANCH @ 10...17
        constr: A @ 10...11
        body: 1 @ 15...16
    "###);
}

#[test]
fn match1_var() {
    insta::assert_debug_snapshot!(parse("match x { A(y) => 1, }"), @r###"
    MATCH
      scrut: x @ 6...7
      branch: BRANCH @ 10...20
        constr: A @ 10...11
        binder: y @ 12...13
        body: 1 @ 18...19
    "###);
}

#[test]
fn match1_block() {
    insta::assert_debug_snapshot!(parse("match x { A => { 1 } }"), @r###"
    MATCH
      scrut: x @ 6...7
      branch: BRANCH @ 10...20
        constr: A @ 10...11
        body: 1 @ 17...18
    "###);
}

#[test]
fn match1_expr_nocomma() {
    insta::assert_debug_snapshot!(parse_err("match x { A => 1 }"), @r###"
    (
        Some(
            ERROR,
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
            MATCH
              scrut: x @ 6...7
              branch: BRANCH @ 10...21
                constr: A @ 10...11
                body: ERROR @ 15...20,
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
    insta::assert_debug_snapshot!(parse("match x { A => 1, B => 2, }"), @r###"
    MATCH
      scrut: x @ 6...7
      branch: BRANCH @ 10...17
        constr: A @ 10...11
        body: 1 @ 15...16
      branch: BRANCH @ 18...25
        constr: B @ 18...19
        body: 2 @ 23...24
    "###);
}

#[test]
fn match2_expr_block() {
    insta::assert_debug_snapshot!(parse("match x { A => 1, B => { 2 } }"), @r###"
    MATCH
      scrut: x @ 6...7
      branch: BRANCH @ 10...17
        constr: A @ 10...11
        body: 1 @ 15...16
      branch: BRANCH @ 18...28
        constr: B @ 18...19
        body: 2 @ 25...26
    "###);
}

#[test]
fn match2_block_expr() {
    insta::assert_debug_snapshot!(parse("match x { A => { 1 } B => 2, }"), @r###"
    MATCH
      scrut: x @ 6...7
      branch: BRANCH @ 10...20
        constr: A @ 10...11
        body: 1 @ 17...18
      branch: BRANCH @ 21...28
        constr: B @ 21...22
        body: 2 @ 26...27
    "###);
}

#[test]
fn match2_blocks() {
    insta::assert_debug_snapshot!(parse("match x { A => { 1 } B => { 2 } }"), @r###"
    MATCH
      scrut: x @ 6...7
      branch: BRANCH @ 10...20
        constr: A @ 10...11
        body: 1 @ 17...18
      branch: BRANCH @ 21...31
        constr: B @ 21...22
        body: 2 @ 28...29
    "###);
}
