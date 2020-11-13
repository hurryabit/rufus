mod decl;
mod expr;
mod type_;

use crate::*;
use syntax::*;

fn parse(input: &str) -> Module {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    module
}

#[test]
fn module() {
    insta::assert_debug_snapshot!(parse(r#"
        type Mono = Int
        fn mono(x: Int) -> Mono { x }
        type Poly<A> = A
        fn poly<A>(x: A) -> Poly<A> { x }
        "#), @r###"
    MODULE
      decl: TYPEDECL
        name: Mono @ 14...18
        type: Int @ 21...24
      decl: FUNCDECL
        name: mono @ 36...40
        param: x @ 41...42
        type: Int @ 44...47
        result: Mono @ 52...56
        body: x @ 57...62
      decl: TYPEDECL
        name: Poly @ 76...80
        type_param: A @ 81...82
        type: A @ 86...87
      decl: FUNCDECL
        name: poly @ 99...103
        type_param: A @ 104...105
        param: x @ 107...108
        type: A @ 110...111
        result: APP @ 116...123
          syn: Poly @ 116...120
          type_arg: A @ 121...122
        body: x @ 124...129
    "###);
}
