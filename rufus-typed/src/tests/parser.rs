mod decl;
mod expr;
mod type_;

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
        type Poly<A> = A
        fn poly<A>(x: A) -> Poly<A> { x }
        "#), @r###"
    Module {
        decls: [
            Type(
                TypeDecl {
                    name: TypeVar(
                        "Mono",
                    ),
                    params: [],
                    body: Var(
                        TypeVar(
                            "Int",
                        ),
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
                            Var(
                                TypeVar(
                                    "Int",
                                ),
                            ),
                        ),
                    ],
                    return_type: Var(
                        TypeVar(
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
                    name: TypeVar(
                        "Poly",
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
            ),
            Func(
                FuncDecl {
                    name: ExprVar(
                        "poly",
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
                    return_type: App(
                        Var(
                            TypeVar(
                                "Poly",
                            ),
                        ),
                        [
                            Var(
                                TypeVar(
                                    "A",
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
