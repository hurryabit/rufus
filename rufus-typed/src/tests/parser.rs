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
    insta::assert_yaml_snapshot!(parse(r#"
        type Mono = Int
        fn mono(x: Int) -> Mono { x }
        type Poly<A> = A
        fn poly<A>(x: A) -> Poly<A> { x }
        "#), @r###"
    ---
    decls:
      - Type:
          name: Mono
          params: []
          body:
            Var: Int
      - Func:
          name: mono
          type_params: []
          expr_params:
            - - x
              - Var: Int
          return_type:
            Var: Mono
          body:
            Var: x
      - Type:
          name: Poly
          params:
            - A
          body:
            Var: A
      - Func:
          name: poly
          type_params:
            - A
          expr_params:
            - - x
              - Var: A
          return_type:
            SynApp:
              - Poly
              - - Var: A
          body:
            Var: x
    "###);
}
