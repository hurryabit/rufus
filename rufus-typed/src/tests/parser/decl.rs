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
      name:
        locatee: T
        span:
          start: 5
          end: 6
      params: []
      body:
        locatee:
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
fn type_poly() {
    insta::assert_yaml_snapshot!(parse("type T<A> = A"), @r###"
    ---
    Type:
      name:
        locatee: T
        span:
          start: 5
          end: 6
      params:
        - locatee: A
          span:
            start: 7
            end: 8
      body:
        locatee:
          Var:
            locatee: A
            span:
              start: 12
              end: 13
        span:
          start: 12
          end: 13
    "###);
}

#[test]
fn func_mono() {
    insta::assert_yaml_snapshot!(parse("fn id(x: Int) -> Int { x }"), @r###"
    ---
    Func:
      name:
        locatee: id
        span:
          start: 3
          end: 5
      type_params: []
      expr_params:
        - - locatee: x
            span:
              start: 6
              end: 7
          - locatee:
              Var:
                locatee: Int
                span:
                  start: 9
                  end: 12
            span:
              start: 9
              end: 12
      return_type:
        locatee:
          Var:
            locatee: Int
            span:
              start: 17
              end: 20
        span:
          start: 17
          end: 20
      body:
        locatee:
          Var:
            locatee: x
            span:
              start: 23
              end: 24
        span:
          start: 21
          end: 26
    "###);
}

#[test]
fn func_poly() {
    insta::assert_yaml_snapshot!(parse("fn id<A>(x: A) -> A { x }"), @r###"
    ---
    Func:
      name:
        locatee: id
        span:
          start: 3
          end: 5
      type_params:
        - locatee: A
          span:
            start: 6
            end: 7
      expr_params:
        - - locatee: x
            span:
              start: 9
              end: 10
          - locatee:
              Var:
                locatee: A
                span:
                  start: 12
                  end: 13
            span:
              start: 12
              end: 13
      return_type:
        locatee:
          Var:
            locatee: A
            span:
              start: 18
              end: 19
        span:
          start: 18
          end: 19
      body:
        locatee:
          Var:
            locatee: x
            span:
              start: 22
              end: 23
        span:
          start: 20
          end: 25
    "###);
}
