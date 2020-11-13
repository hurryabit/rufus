use crate::*;
use check::LError;
use syntax::Module;

fn check(input: &str) -> Module {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    module.check().unwrap();
    module
}

fn check_err(input: &str) -> LError {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    module.check().unwrap_err()
}

#[test]
fn unknown_type_var() {
    insta::assert_debug_snapshot!(check_err("type Bad = Unknown"), @r###"
    Located {
        locatee: UnknownTypeVar(
            t#Unknown,
        ),
        span: Span {
            start: 11,
            end: 18,
        },
    }
    "###);
}

#[test]
fn unexpected_type_con_at_top() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = Id"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Id,
            expected: 0,
            found: 1,
        },
        span: Span {
            start: 26,
            end: 28,
        },
    }
    "###);
}

#[test]
fn unexpected_type_con_in_type_args() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype List<A> = A\ntype Bad = List<Id>"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Id,
            expected: 0,
            found: 1,
        },
        span: Span {
            start: 48,
            end: 50,
        },
    }
    "###);
}

#[test]
fn unexpected_type_con_in_func_args() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = (Id) -> Int"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Id,
            expected: 0,
            found: 1,
        },
        span: Span {
            start: 27,
            end: 29,
        },
    }
    "###);
}

#[test]
fn unexpected_type_con_in_func_result() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = () -> Id"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Id,
            expected: 0,
            found: 1,
        },
        span: Span {
            start: 32,
            end: 34,
        },
    }
    "###);
}

#[test]
fn unexpected_type_con_in_record() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = {field: Id}"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Id,
            expected: 0,
            found: 1,
        },
        span: Span {
            start: 34,
            end: 36,
        },
    }
    "###);
}

#[test]
fn unexpected_type_con_in_variant() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = [Constr(Id)]"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Id,
            expected: 0,
            found: 1,
        },
        span: Span {
            start: 34,
            end: 36,
        },
    }
    "###);
}

#[test]
fn wrong_arity_var() {
    insta::assert_debug_snapshot!(check_err("type Bad<F> = F<Int>"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#F,
            expected: 1,
            found: 0,
        },
        span: Span {
            start: 14,
            end: 20,
        },
    }
    "###);
}

#[test]
fn wrong_arity_builtin() {
    insta::assert_debug_snapshot!(check_err("type Bad<A> = Int<A>"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Int,
            expected: 1,
            found: 0,
        },
        span: Span {
            start: 14,
            end: 20,
        },
    }
    "###);
}

#[test]
fn wrong_arity_type_syn() {
    insta::assert_debug_snapshot!(check_err("type Syn = Int\ntype Bad = Syn<Int>"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Syn,
            expected: 1,
            found: 0,
        },
        span: Span {
            start: 26,
            end: 34,
        },
    }
    "###);
}

#[test]
fn wrong_arity_type_con_syn() {
    insta::assert_debug_snapshot!(check_err("type Syn<A> = A\ntype Bad = Syn<Int, Int>"), @r###"
    Located {
        locatee: KindMismatch {
            type_var: t#Syn,
            expected: 2,
            found: 1,
        },
        span: Span {
            start: 27,
            end: 40,
        },
    }
    "###);
}

#[test]
fn int_resolved() {
    insta::assert_yaml_snapshot!(check("type Here = Int"), @r###"
    ---
    decls:
      - Type:
          name:
            locatee: Here
            span:
              start: 5
              end: 9
          params: []
          body:
            locatee: Int
            span:
              start: 12
              end: 15
    "###);
}

#[test]
fn bool_resolved() {
    insta::assert_yaml_snapshot!(check("type Here = Bool"), @r###"
    ---
    decls:
      - Type:
          name:
            locatee: Here
            span:
              start: 5
              end: 9
          params: []
          body:
            locatee: Bool
            span:
              start: 12
              end: 16
    "###);
}

#[test]
fn syn_resolved() {
    insta::assert_yaml_snapshot!(check("type Syn = Int\ntype Here = Syn"), @r###"
    ---
    decls:
      - Type:
          name:
            locatee: Syn
            span:
              start: 5
              end: 8
          params: []
          body:
            locatee: Int
            span:
              start: 11
              end: 14
      - Type:
          name:
            locatee: Here
            span:
              start: 20
              end: 24
          params: []
          body:
            locatee:
              SynApp:
                - locatee: Syn
                  span:
                    start: 27
                    end: 30
                - []
            span:
              start: 27
              end: 30
    "###);
}

#[test]
fn var_resolved() {
    insta::assert_yaml_snapshot!(check("type Here<A> = A"), @r###"
    ---
    decls:
      - Type:
          name:
            locatee: Here
            span:
              start: 5
              end: 9
          params:
            - locatee: A
              span:
                start: 10
                end: 11
          body:
            locatee:
              Var: A
            span:
              start: 15
              end: 16
    "###);
}

#[test]
fn var_shadows_int() {
    insta::assert_yaml_snapshot!(check("type Here<Int> = Int"), @r###"
    ---
    decls:
      - Type:
          name:
            locatee: Here
            span:
              start: 5
              end: 9
          params:
            - locatee: Int
              span:
                start: 10
                end: 13
          body:
            locatee:
              Var: Int
            span:
              start: 17
              end: 20
    "###);
}

#[test]
fn type_syn_shadows_int() {
    insta::assert_yaml_snapshot!(check("type Int = Bool\ntype Here = Int"), @r###"
    ---
    decls:
      - Type:
          name:
            locatee: Int
            span:
              start: 5
              end: 8
          params: []
          body:
            locatee: Bool
            span:
              start: 11
              end: 15
      - Type:
          name:
            locatee: Here
            span:
              start: 21
              end: 25
          params: []
          body:
            locatee:
              SynApp:
                - locatee: Int
                  span:
                    start: 28
                    end: 31
                - []
            span:
              start: 28
              end: 31
    "###);
}

#[test]
fn type_con_syn_shadows_int() {
    insta::assert_yaml_snapshot!(check("type Int<A> = A\ntype Here = Int<Bool>"), @r###"
    ---
    decls:
      - Type:
          name:
            locatee: Int
            span:
              start: 5
              end: 8
          params:
            - locatee: A
              span:
                start: 9
                end: 10
          body:
            locatee:
              Var: A
            span:
              start: 14
              end: 15
      - Type:
          name:
            locatee: Here
            span:
              start: 21
              end: 25
          params: []
          body:
            locatee:
              SynApp:
                - locatee: Int
                  span:
                    start: 28
                    end: 31
                - - locatee: Bool
                    span:
                      start: 32
                      end: 36
            span:
              start: 28
              end: 37
    "###);
}
