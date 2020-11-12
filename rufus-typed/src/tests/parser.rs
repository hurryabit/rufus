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
          name:
            locatee: Mono
            span:
              start: 14
              end: 18
          params: []
          body:
            locatee:
              Var:
                locatee: Int
                span:
                  start: 21
                  end: 24
            span:
              start: 21
              end: 24
      - Func:
          name:
            locatee: mono
            span:
              start: 36
              end: 40
          type_params: []
          expr_params:
            - - locatee: x
                span:
                  start: 41
                  end: 42
              - locatee:
                  Var:
                    locatee: Int
                    span:
                      start: 44
                      end: 47
                span:
                  start: 44
                  end: 47
          return_type:
            locatee:
              Var:
                locatee: Mono
                span:
                  start: 52
                  end: 56
            span:
              start: 52
              end: 56
          body:
            locatee:
              Var:
                locatee: x
                span:
                  start: 59
                  end: 60
            span:
              start: 57
              end: 62
      - Type:
          name:
            locatee: Poly
            span:
              start: 76
              end: 80
          params:
            - locatee: A
              span:
                start: 81
                end: 82
          body:
            locatee:
              Var:
                locatee: A
                span:
                  start: 86
                  end: 87
            span:
              start: 86
              end: 87
      - Func:
          name:
            locatee: poly
            span:
              start: 99
              end: 103
          type_params:
            - locatee: A
              span:
                start: 104
                end: 105
          expr_params:
            - - locatee: x
                span:
                  start: 107
                  end: 108
              - locatee:
                  Var:
                    locatee: A
                    span:
                      start: 110
                      end: 111
                span:
                  start: 110
                  end: 111
          return_type:
            locatee:
              SynApp:
                - locatee: Poly
                  span:
                    start: 116
                    end: 120
                - - locatee:
                      Var:
                        locatee: A
                        span:
                          start: 121
                          end: 122
                    span:
                      start: 121
                      end: 122
            span:
              start: 116
              end: 123
          body:
            locatee:
              Var:
                locatee: x
                span:
                  start: 126
                  end: 127
            span:
              start: 124
              end: 129
    "###);
}
