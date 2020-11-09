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
            name: TypeVar(
                "T",
            ),
            params: [],
            body: Var(
                TypeVar(
                    "Int",
                ),
            ),
        },
    )
    "###);
}

#[test]
fn type_poly() {
    insta::assert_debug_snapshot!(parse("type T<A> = A"), @r###"
    Type(
        TypeDecl {
            name: TypeVar(
                "T",
            ),
            params: [
                TypeVar(
                    "A",
                ),
            ],
            body: Var(
                TypeVar(
                    "A",
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
                    Var(
                        TypeVar(
                            "Int",
                        ),
                    ),
                ),
            ],
            return_type: Var(
                TypeVar(
                    "Int",
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

#[test]
fn func_poly() {
    insta::assert_debug_snapshot!(parse("fn id<A>(x: A) -> A { x }"), @r###"
    Func(
        FuncDecl {
            name: ExprVar(
                "id",
            ),
            type_params: [
                TypeVar(
                    "A",
                ),
            ],
            expr_params: [
                (
                    ExprVar(
                        "x",
                    ),
                    Var(
                        TypeVar(
                            "A",
                        ),
                    ),
                ),
            ],
            return_type: Var(
                TypeVar(
                    "A",
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
