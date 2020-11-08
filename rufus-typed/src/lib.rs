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
                ("A<Int,Bool>", App(TypeCon::new("A"), vec![Int, Bool])),
                ("{}", Record(vec![])),
                ("{a: Int}", Record(vec![(ExprVar::new("a"), Int)])),
                ("{a: Int,}", Record(vec![(ExprVar::new("a"), Int)])),
                (
                    "[A | B(Int)]",
                    Variant(vec![
                        (ExprCon::new("A"), None),
                        (ExprCon::new("B"), Some(Int)),
                    ]),
                ),
                (
                    "[Int(Int)]",
                    Variant(vec![(ExprCon::new("Int"), Some(Int))]),
                ),
                (
                    "[Bool(Bool)]",
                    Variant(vec![(ExprCon::new("Bool"), Some(Bool))]),
                ),
                // TODO(MH): We want to allow an optional leading "|" rather
                // than a trailing one.
                (
                    "[A | B(Int) |]",
                    Variant(vec![
                        (ExprCon::new("A"), None),
                        (ExprCon::new("B"), Some(Int)),
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

        mod expr {
            use crate::*;
            use syntax::*;

            fn parse(input: &str) -> Expr {
                let parser = parser::ExprParser::new();
                parser.parse(input).unwrap()
            }

            fn parse_block(input: &str) -> Expr {
                let parser = parser::BlockExprParser::new();
                parser.parse(input).unwrap()
            }

            fn parse_err(input: &str) -> String {
                let parser = parser::ExprParser::new();
                parser.parse(input).unwrap_err().to_string()
            }

            #[test]
            fn var() {
                insta::assert_debug_snapshot!(parse("x"), @r###"
                Var(
                    ExprVar(
                        "x",
                    ),
                )
                "###);
            }

            #[test]
            fn num() {
                insta::assert_debug_snapshot!(parse("0"), @r###"
                Num(
                    0,
                )
                "###);
            }

            #[test]
            fn bool_true() {
                insta::assert_debug_snapshot!(parse("true"), @r###"
                Bool(
                    true,
                )
                "###);
            }

            #[test]
            fn bool_false() {
                insta::assert_debug_snapshot!(parse("false"), @r###"
                Bool(
                    false,
                )
                "###);
            }

            #[test]
            fn app0() {
                insta::assert_debug_snapshot!(parse("f()"), @r###"
                App(
                    Var(
                        ExprVar(
                            "f",
                        ),
                    ),
                    [],
                )
                "###);
            }

            #[test]
            fn app1() {
                insta::assert_debug_snapshot!(parse("f(1)"), @r###"
                App(
                    Var(
                        ExprVar(
                            "f",
                        ),
                    ),
                    [
                        Num(
                            1,
                        ),
                    ],
                )
                "###);
            }

            #[test]
            fn app1_trailing() {
                insta::assert_debug_snapshot!(parse("f(1,)"), @r###"
                App(
                    Var(
                        ExprVar(
                            "f",
                        ),
                    ),
                    [
                        Num(
                            1,
                        ),
                    ],
                )
                "###);
            }

            #[test]
            fn app2() {
                insta::assert_debug_snapshot!(parse("f(1, 2)"), @r###"
                App(
                    Var(
                        ExprVar(
                            "f",
                        ),
                    ),
                    [
                        Num(
                            1,
                        ),
                        Num(
                            2,
                        ),
                    ],
                )
                "###);
            }
            #[test]
            fn app_ty() {
                insta::assert_debug_snapshot!(parse("f@<Int>(1)"), @r###"
                App(
                    TypeApp(
                        Var(
                            ExprVar(
                                "f",
                            ),
                        ),
                        [
                            Int,
                        ],
                    ),
                    [
                        Num(
                            1,
                        ),
                    ],
                )
                "###);
            }

            #[test]
            fn app_ty_err() {
                insta::assert_debug_snapshot!(parse_err("f<A>(1)"), @r###""Unrecognized token `>` found at 3:4\nExpected one of \")\", \"+\", \",\", \"-\", \";\", \"{\" or \"}\"""###);
            }

            #[test]
            fn record0() {
                insta::assert_debug_snapshot!(parse("{}"), @r###"
                Record(
                    [],
                )
                "###);
            }
            #[test]
            fn record1() {
                insta::assert_debug_snapshot!(parse("{x = 1}"), @r###"
                Record(
                    [
                        (
                            ExprVar(
                                "x",
                            ),
                            Num(
                                1,
                            ),
                        ),
                    ],
                )
                "###);
            }

            #[test]
            fn record1_trailing() {
                insta::assert_debug_snapshot!(parse("{x = 1,}"), @r###"
                Record(
                    [
                        (
                            ExprVar(
                                "x",
                            ),
                            Num(
                                1,
                            ),
                        ),
                    ],
                )
                "###);
            }

            #[test]
            fn record2() {
                insta::assert_debug_snapshot!(parse("{x = 1, y = 2}"), @r###"
                Record(
                    [
                        (
                            ExprVar(
                                "x",
                            ),
                            Num(
                                1,
                            ),
                        ),
                        (
                            ExprVar(
                                "y",
                            ),
                            Num(
                                2,
                            ),
                        ),
                    ],
                )
                "###);
            }

            #[test]
            fn proj1() {
                insta::assert_debug_snapshot!(parse("r.x"), @r###"
                Proj(
                    Var(
                        ExprVar(
                            "r",
                        ),
                    ),
                    ExprVar(
                        "x",
                    ),
                )
                "###);
            }

            #[test]
            fn proj2() {
                insta::assert_debug_snapshot!(parse("r.x.y"), @r###"
                Proj(
                    Proj(
                        Var(
                            ExprVar(
                                "r",
                            ),
                        ),
                        ExprVar(
                            "x",
                        ),
                    ),
                    ExprVar(
                        "y",
                    ),
                )
                "###);
            }

            #[test]
            fn variant0() {
                insta::assert_debug_snapshot!(parse("A"), @r###"
                Variant(
                    ExprCon(
                        "A",
                    ),
                    None,
                )
                "###);
            }

            #[test]
            fn variant1() {
                insta::assert_debug_snapshot!(parse("A(0)"), @r###"
                Variant(
                    ExprCon(
                        "A",
                    ),
                    Some(
                        Num(
                            0,
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn variant_int() {
                insta::assert_debug_snapshot!(parse("Int"), @r###"
                Variant(
                    ExprCon(
                        "Int",
                    ),
                    None,
                )
                "###);
            }

            #[test]
            fn variant_bool() {
                insta::assert_debug_snapshot!(parse("Bool"), @r###"
                Variant(
                    ExprCon(
                        "Bool",
                    ),
                    None,
                )
                "###);
            }

            #[test]
            fn prod2() {
                insta::assert_debug_snapshot!(parse("a*b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    Mul,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn prod3() {
                insta::assert_debug_snapshot!(parse("a/b*c"), @r###"
                BinOp(
                    BinOp(
                        Var(
                            ExprVar(
                                "a",
                            ),
                        ),
                        Div,
                        Var(
                            ExprVar(
                                "b",
                            ),
                        ),
                    ),
                    Mul,
                    Var(
                        ExprVar(
                            "c",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn sum2() {
                insta::assert_debug_snapshot!(parse("a+b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    Add,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn sum3() {
                insta::assert_debug_snapshot!(parse("a-b+c"), @r###"
                BinOp(
                    BinOp(
                        Var(
                            ExprVar(
                                "a",
                            ),
                        ),
                        Sub,
                        Var(
                            ExprVar(
                                "b",
                            ),
                        ),
                    ),
                    Add,
                    Var(
                        ExprVar(
                            "c",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_eq() {
                insta::assert_debug_snapshot!(parse("a == b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    Equals,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_neq() {
                insta::assert_debug_snapshot!(parse("a != b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    NotEq,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_lt() {
                insta::assert_debug_snapshot!(parse("a < b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    Less,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_leq() {
                insta::assert_debug_snapshot!(parse("a <= b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    LessEq,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_gt() {
                insta::assert_debug_snapshot!(parse("a > b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    Greater,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_geq() {
                insta::assert_debug_snapshot!(parse("a >= b"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    GreaterEq,
                    Var(
                        ExprVar(
                            "b",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_mixed() {
                insta::assert_debug_snapshot!(parse("a + b == c * d"), @r###"
                BinOp(
                    BinOp(
                        Var(
                            ExprVar(
                                "a",
                            ),
                        ),
                        Add,
                        Var(
                            ExprVar(
                                "b",
                            ),
                        ),
                    ),
                    Equals,
                    BinOp(
                        Var(
                            ExprVar(
                                "c",
                            ),
                        ),
                        Mul,
                        Var(
                            ExprVar(
                                "d",
                            ),
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_many() {
                insta::assert_debug_snapshot!(parse("a == (b == c)"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    Equals,
                    BinOp(
                        Var(
                            ExprVar(
                                "b",
                            ),
                        ),
                        Equals,
                        Var(
                            ExprVar(
                                "c",
                            ),
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn cmp_many_err() {
                insta::assert_debug_snapshot!(parse_err("a == b == c"), @r###""Unrecognized token `==` found at 7:9\nExpected one of \")\", \"+\", \",\", \"-\", \";\", \"{\" or \"}\"""###);
            }

            #[test]
            fn sum_prod() {
                insta::assert_debug_snapshot!(parse("a+b*c"), @r###"
                BinOp(
                    Var(
                        ExprVar(
                            "a",
                        ),
                    ),
                    Add,
                    BinOp(
                        Var(
                            ExprVar(
                                "b",
                            ),
                        ),
                        Mul,
                        Var(
                            ExprVar(
                                "c",
                            ),
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn lam0() {
                insta::assert_debug_snapshot!(parse("fn() { 0 }"), @r###"
            Lam(
                [],
                Num(
                    0,
                ),
            )
            "###);
            }

            #[test]
            fn lam1() {
                insta::assert_debug_snapshot!(parse("fn(x) { x }"), @r###"
            Lam(
                [
                    (
                        ExprVar(
                            "x",
                        ),
                        None,
                    ),
                ],
                Var(
                    ExprVar(
                        "x",
                    ),
                ),
            )
            "###);
            }

            #[test]
            fn lam1_trailing() {
                insta::assert_debug_snapshot!(parse("fn(x,) { x }"), @r###"
            Lam(
                [
                    (
                        ExprVar(
                            "x",
                        ),
                        None,
                    ),
                ],
                Var(
                    ExprVar(
                        "x",
                    ),
                ),
            )
            "###);
            }

            #[test]
            fn lam2() {
                insta::assert_debug_snapshot!(parse("fn(x, y) { x }"), @r###"
            Lam(
                [
                    (
                        ExprVar(
                            "x",
                        ),
                        None,
                    ),
                    (
                        ExprVar(
                            "y",
                        ),
                        None,
                    ),
                ],
                Var(
                    ExprVar(
                        "x",
                    ),
                ),
            )
            "###);
            }
            #[test]
            fn lam1_typed() {
                insta::assert_debug_snapshot!(parse("fn(x: Int) { x }"), @r###"
            Lam(
                [
                    (
                        ExprVar(
                            "x",
                        ),
                        Some(
                            Int,
                        ),
                    ),
                ],
                Var(
                    ExprVar(
                        "x",
                    ),
                ),
            )
            "###);
            }

            #[test]
            fn lam1_poly() {
                insta::assert_debug_snapshot!(parse("fn<a>(x: a) { x }"), @r###"
            TypeAbs(
                [
                    TypeVar(
                        "a",
                    ),
                ],
                Lam(
                    [
                        (
                            ExprVar(
                                "x",
                            ),
                            Some(
                                Var(
                                    TypeVar(
                                        "a",
                                    ),
                                ),
                            ),
                        ),
                    ],
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                ),
            )
            "###);
            }

            #[test]
            fn if_atom() {
                insta::assert_debug_snapshot!(parse("if true { 0 } else { 1 }"), @r###"
                If(
                    Bool(
                        true,
                    ),
                    Num(
                        0,
                    ),
                    Num(
                        1,
                    ),
                )
                "###);
            }

            #[test]
            fn if_cmp() {
                insta::assert_debug_snapshot!(parse("if a == b { 0 } else { 1 }"), @r###"
                If(
                    BinOp(
                        Var(
                            ExprVar(
                                "a",
                            ),
                        ),
                        Equals,
                        Var(
                            ExprVar(
                                "b",
                            ),
                        ),
                    ),
                    Num(
                        0,
                    ),
                    Num(
                        1,
                    ),
                )
                "###);
            }

            #[test]
            fn block_atom() {
                insta::assert_debug_snapshot!(parse_block("{ a }"), @r###"
                Var(
                    ExprVar(
                        "a",
                    ),
                )
                "###);
            }

            #[test]
            fn block_record() {
                insta::assert_debug_snapshot!(parse_block("{ {f = 1} }"), @r###"
                Record(
                    [
                        (
                            ExprVar(
                                "f",
                            ),
                            Num(
                                1,
                            ),
                        ),
                    ],
                )
                "###);
            }

            #[test]
            fn let1_atom() {
                insta::assert_debug_snapshot!(parse_block("{ let x = 1; x }"), @r###"
                Let(
                    ExprVar(
                        "x",
                    ),
                    None,
                    Num(
                        1,
                    ),
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn let1_complex() {
                insta::assert_debug_snapshot!(parse_block("{ let x = 1 + 1; x }"), @r###"
                Let(
                    ExprVar(
                        "x",
                    ),
                    None,
                    BinOp(
                        Num(
                            1,
                        ),
                        Add,
                        Num(
                            1,
                        ),
                    ),
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn let1_typed() {
                insta::assert_debug_snapshot!(parse_block("{ let x: Int = 1; x }"), @r###"
                Let(
                    ExprVar(
                        "x",
                    ),
                    Some(
                        Int,
                    ),
                    Num(
                        1,
                    ),
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn let1_block() {
                insta::assert_debug_snapshot!(parse_block("{ let x = { 1 }; x }"), @r###"
                Let(
                    ExprVar(
                        "x",
                    ),
                    None,
                    Num(
                        1,
                    ),
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn let2() {
                insta::assert_debug_snapshot!(parse_block("{ let x = 1; let y = x; y }"), @r###"
                Let(
                    ExprVar(
                        "x",
                    ),
                    None,
                    Num(
                        1,
                    ),
                    Let(
                        ExprVar(
                            "y",
                        ),
                        None,
                        Var(
                            ExprVar(
                                "x",
                            ),
                        ),
                        Var(
                            ExprVar(
                                "y",
                            ),
                        ),
                    ),
                )
                "###);
            }

            #[test]
            fn match1_novar() {
                insta::assert_debug_snapshot!(parse("match x { A => 1, }"), @r###"
                Match(
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                    [
                        Branch {
                            con: ExprCon(
                                "A",
                            ),
                            var: None,
                            rhs: Num(
                                1,
                            ),
                        },
                    ],
                )
                "###);
            }

            #[test]
            fn match1_var() {
                insta::assert_debug_snapshot!(parse("match x { A(y) => 1, }"), @r###"
                Match(
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                    [
                        Branch {
                            con: ExprCon(
                                "A",
                            ),
                            var: Some(
                                ExprVar(
                                    "y",
                                ),
                            ),
                            rhs: Num(
                                1,
                            ),
                        },
                    ],
                )
                "###);
            }

            #[test]
            fn match1_block() {
                insta::assert_debug_snapshot!(parse("match x { A => { 1 } }"), @r###"
                Match(
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                    [
                        Branch {
                            con: ExprCon(
                                "A",
                            ),
                            var: None,
                            rhs: Num(
                                1,
                            ),
                        },
                    ],
                )
                "###);
            }

            #[test]
            fn match1_expr_nocomma() {
                insta::assert_debug_snapshot!(parse_err("match x { A => 1 }"), @r###""Unrecognized token `}` found at 17:18\nExpected one of \",\"""###);
            }

            #[test]
            fn match1_block_comma() {
                insta::assert_debug_snapshot!(parse_err("match x { A => { 1 }, }"), @r###""Unrecognized token `,` found at 20:21\nExpected one of \"Bool\", \"Int\", \"}\" or r#\"[A-Z]\\\\w*\"#""###);
            }

            #[test]
            fn match2_exprs() {
                insta::assert_debug_snapshot!(parse("match x { A => 1, B => 2, }"), @r###"
                Match(
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                    [
                        Branch {
                            con: ExprCon(
                                "A",
                            ),
                            var: None,
                            rhs: Num(
                                1,
                            ),
                        },
                        Branch {
                            con: ExprCon(
                                "B",
                            ),
                            var: None,
                            rhs: Num(
                                2,
                            ),
                        },
                    ],
                )
                "###);
            }

            #[test]
            fn match2_expr_block() {
                insta::assert_debug_snapshot!(parse("match x { A => 1, B => { 2 } }"), @r###"
                Match(
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                    [
                        Branch {
                            con: ExprCon(
                                "A",
                            ),
                            var: None,
                            rhs: Num(
                                1,
                            ),
                        },
                        Branch {
                            con: ExprCon(
                                "B",
                            ),
                            var: None,
                            rhs: Num(
                                2,
                            ),
                        },
                    ],
                )
                "###);
            }

            #[test]
            fn match2_block_expr() {
                insta::assert_debug_snapshot!(parse("match x { A => { 1 } B => 2, }"), @r###"
                Match(
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                    [
                        Branch {
                            con: ExprCon(
                                "A",
                            ),
                            var: None,
                            rhs: Num(
                                1,
                            ),
                        },
                        Branch {
                            con: ExprCon(
                                "B",
                            ),
                            var: None,
                            rhs: Num(
                                2,
                            ),
                        },
                    ],
                )
                "###);
            }

            #[test]
            fn match2_blocks() {
                insta::assert_debug_snapshot!(parse("match x { A => { 1 } B => { 2 } }"), @r###"
                Match(
                    Var(
                        ExprVar(
                            "x",
                        ),
                    ),
                    [
                        Branch {
                            con: ExprCon(
                                "A",
                            ),
                            var: None,
                            rhs: Num(
                                1,
                            ),
                        },
                        Branch {
                            con: ExprCon(
                                "B",
                            ),
                            var: None,
                            rhs: Num(
                                2,
                            ),
                        },
                    ],
                )
                "###);
            }
        }

        mod decl {
            use crate::*;
            use syntax::*;

            fn parse(input: &str) -> Decl {
                let parser = parser::DeclParser::new();
                parser.parse(input).unwrap()
            }

            #[test]
            fn type_mono() {
                insta::assert_debug_snapshot!(parse("type T = Int"), @r###"
                Type(
                    TypeDecl {
                        name: TypeCon(
                            "T",
                        ),
                        body: Abs(
                            [],
                            Int,
                        ),
                    },
                )
                "###);
            }

            #[test]
            fn type_poly() {
                insta::assert_debug_snapshot!(parse("type T<a> = a"), @r###"
                Type(
                    TypeDecl {
                        name: TypeCon(
                            "T",
                        ),
                        body: Abs(
                            [
                                TypeVar(
                                    "a",
                                ),
                            ],
                            Var(
                                TypeVar(
                                    "a",
                                ),
                            ),
                        ),
                    },
                )
                "###);
            }

            #[test]
            fn func_mono() {
                insta::assert_debug_snapshot!(parse("fn id(x: Int) -> Int { x }"), @r###"
                Func(
                    FuncDecl {
                        name: ExprVar(
                            "id",
                        ),
                        type_params: [],
                        expr_params: [
                            (
                                ExprVar(
                                    "x",
                                ),
                                Int,
                            ),
                        ],
                        return_type: Int,
                        body: Var(
                            ExprVar(
                                "x",
                            ),
                        ),
                    },
                )
                "###);
            }

            #[test]
            fn func_poly() {
                insta::assert_debug_snapshot!(parse("fn id<a>(x: a) -> a { x }"), @r###"
                Func(
                    FuncDecl {
                        name: ExprVar(
                            "id",
                        ),
                        type_params: [
                            TypeVar(
                                "a",
                            ),
                        ],
                        expr_params: [
                            (
                                ExprVar(
                                    "x",
                                ),
                                Var(
                                    TypeVar(
                                        "a",
                                    ),
                                ),
                            ),
                        ],
                        return_type: Var(
                            TypeVar(
                                "a",
                            ),
                        ),
                        body: Var(
                            ExprVar(
                                "x",
                            ),
                        ),
                    },
                )
                "###);
            }
        }

        mod module {
            use crate::*;
            use syntax::*;

            fn parse(input: &str) -> Module {
                let parser = parser::ModuleParser::new();
                parser.parse(input).unwrap()
            }

            #[test]
            fn module() {
                insta::assert_debug_snapshot!(parse(r#"
                type Mono = Int

                fn mono(x: Int) -> Mono { x }

                type Poly<a> = a

                fn poly<a>(x: a) -> Poly<a> { x }
                "#), @r###"
                Module {
                    decls: [
                        Type(
                            TypeDecl {
                                name: TypeCon(
                                    "Mono",
                                ),
                                body: Abs(
                                    [],
                                    Int,
                                ),
                            },
                        ),
                        Func(
                            FuncDecl {
                                name: ExprVar(
                                    "mono",
                                ),
                                type_params: [],
                                expr_params: [
                                    (
                                        ExprVar(
                                            "x",
                                        ),
                                        Int,
                                    ),
                                ],
                                return_type: Synonym(
                                    TypeCon(
                                        "Mono",
                                    ),
                                ),
                                body: Var(
                                    ExprVar(
                                        "x",
                                    ),
                                ),
                            },
                        ),
                        Type(
                            TypeDecl {
                                name: TypeCon(
                                    "Poly",
                                ),
                                body: Abs(
                                    [
                                        TypeVar(
                                            "a",
                                        ),
                                    ],
                                    Var(
                                        TypeVar(
                                            "a",
                                        ),
                                    ),
                                ),
                            },
                        ),
                        Func(
                            FuncDecl {
                                name: ExprVar(
                                    "poly",
                                ),
                                type_params: [
                                    TypeVar(
                                        "a",
                                    ),
                                ],
                                expr_params: [
                                    (
                                        ExprVar(
                                            "x",
                                        ),
                                        Var(
                                            TypeVar(
                                                "a",
                                            ),
                                        ),
                                    ),
                                ],
                                return_type: App(
                                    TypeCon(
                                        "Poly",
                                    ),
                                    [
                                        Var(
                                            TypeVar(
                                                "a",
                                            ),
                                        ),
                                    ],
                                ),
                                body: Var(
                                    ExprVar(
                                        "x",
                                    ),
                                ),
                            },
                        ),
                    ],
                }
                "###);
            }
        }
    }
}
