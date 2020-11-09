use crate::*;
use syntax::*;

fn parse(input: &str) -> Decl {
    let parser = parser::DeclParser::new();
    let mut errors = Vec::new();
    let decl = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    decl
}

#[test]
fn type_mono() {
    insta::assert_yaml_snapshot!(parse("type T = Int"), @r###"
    ---
    Type:
      name: T
      params: []
      body:
        Var: Int
    "###);
}

#[test]
fn type_poly() {
    insta::assert_yaml_snapshot!(parse("type T<A> = A"), @r###"
    ---
    Type:
      name: T
      params:
        - A
      body:
        Var: A
    "###);
}

#[test]
fn func_mono() {
    insta::assert_yaml_snapshot!(parse("fn id(x: Int) -> Int { x }"), @r###"
    ---
    Func:
      name: id
      type_params: []
      expr_params:
        - - x
          - Var: Int
      return_type:
        Var: Int
      body:
        Var: x
    "###);
}

#[test]
fn func_poly() {
    insta::assert_yaml_snapshot!(parse("fn id<A>(x: A) -> A { x }"), @r###"
    ---
    Func:
      name: id
      type_params:
        - A
      expr_params:
        - - x
          - Var: A
      return_type:
        Var: A
      body:
        Var: x
    "###);
}
