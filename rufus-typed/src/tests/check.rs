use crate::*;
use check::Error;
use syntax::Module;

fn check(input: &str) -> Module {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    module.check().unwrap();
    module
}

fn check_err(input: &str) -> Error {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    module.check().unwrap_err()
}

#[test]
fn unknown_type_var() {
    insta::assert_debug_snapshot!(check_err("type Bad = Unknown"), @r###"
    UnknownTypeVar(
        TypeVar(
            "Unknown",
        ),
    )
    "###);
}

#[test]
fn unexpected_type_con_at_top() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = Id"), @r###"
    ExpectedTypeFoundTypeCon(
        Syn(
            TypeVar(
                "Id",
            ),
        ),
    )
    "###);
}

#[test]
fn unexpected_type_con_in_type_args() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype List<A> = A\ntype Bad = List<Id>"), @r###"
    ExpectedTypeFoundTypeCon(
        Syn(
            TypeVar(
                "Id",
            ),
        ),
    )
    "###);
}

#[test]
fn unexpected_type_con_in_func_args() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = (Id) -> Int"), @r###"
    ExpectedTypeFoundTypeCon(
        Syn(
            TypeVar(
                "Id",
            ),
        ),
    )
    "###);
}

#[test]
fn unexpected_type_con_in_func_result() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = () -> Id"), @r###"
    ExpectedTypeFoundTypeCon(
        Syn(
            TypeVar(
                "Id",
            ),
        ),
    )
    "###);
}

#[test]
fn unexpected_type_con_in_record() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = {field: Id}"), @r###"
    ExpectedTypeFoundTypeCon(
        Syn(
            TypeVar(
                "Id",
            ),
        ),
    )
    "###);
}

#[test]
fn unexpected_type_con_in_variant() {
    insta::assert_debug_snapshot!(check_err("type Id<A> = A\ntype Bad = [Constr(Id)]"), @r###"
    ExpectedTypeFoundTypeCon(
        Syn(
            TypeVar(
                "Id",
            ),
        ),
    )
    "###);
}

#[test]
fn wrong_arity_var() {
    insta::assert_debug_snapshot!(check_err("type Bad<F> = F<Int>"), @r###"
    WrongNumberOfTypeArgs {
        typ: App(
            Var(
                TypeVar(
                    "F",
                ),
            ),
            [
                Var(
                    TypeVar(
                        "Int",
                    ),
                ),
            ],
        ),
        expected: 0,
        found: 1,
    }
    "###);
}

#[test]
fn wrong_arity_builtin() {
    insta::assert_debug_snapshot!(check_err("type Bad<A> = Int<A>"), @r###"
    WrongNumberOfTypeArgs {
        typ: App(
            Int,
            [
                Var(
                    TypeVar(
                        "A",
                    ),
                ),
            ],
        ),
        expected: 0,
        found: 1,
    }
    "###);
}

#[test]
fn wrong_arity_type_syn() {
    insta::assert_debug_snapshot!(check_err("type Syn = Int\ntype Bad = Syn<Int>"), @r###"
    WrongNumberOfTypeArgs {
        typ: App(
            Syn(
                TypeVar(
                    "Syn",
                ),
            ),
            [
                Var(
                    TypeVar(
                        "Int",
                    ),
                ),
            ],
        ),
        expected: 0,
        found: 1,
    }
    "###);
}

#[test]
fn wrong_arity_type_con_syn() {
    insta::assert_debug_snapshot!(check_err("type Syn<A> = A\ntype Bad = Syn<Int, Int>"), @r###"
    WrongNumberOfTypeArgs {
        typ: App(
            Syn(
                TypeVar(
                    "Syn",
                ),
            ),
            [
                Var(
                    TypeVar(
                        "Int",
                    ),
                ),
                Var(
                    TypeVar(
                        "Int",
                    ),
                ),
            ],
        ),
        expected: 1,
        found: 2,
    }
    "###);
}

#[test]
fn int_resolved() {
    insta::assert_yaml_snapshot!(check("type Here = Int"), @r###"
    ---
    decls:
      - Type:
          name: Here
          params: []
          body: Int
    "###);
}

#[test]
fn bool_resolved() {
    insta::assert_yaml_snapshot!(check("type Here = Bool"), @r###"
    ---
    decls:
      - Type:
          name: Here
          params: []
          body: Bool
    "###);
}

#[test]
fn syn_resolved() {
    insta::assert_yaml_snapshot!(check("type Syn = Int\ntype Here = Syn"), @r###"
    ---
    decls:
      - Type:
          name: Syn
          params: []
          body: Int
      - Type:
          name: Here
          params: []
          body:
            Syn: Syn
    "###);
}

#[test]
fn var_resolved() {
    insta::assert_yaml_snapshot!(check("type Here<A> = A"), @r###"
    ---
    decls:
      - Type:
          name: Here
          params:
            - A
          body:
            Var: A
    "###);
}

#[test]
fn var_shadows_int() {
    insta::assert_yaml_snapshot!(check("type Here<Int> = Int"), @r###"
    ---
    decls:
      - Type:
          name: Here
          params:
            - Int
          body:
            Var: Int
    "###);
}

#[test]
fn type_syn_shadows_int() {
    insta::assert_yaml_snapshot!(check("type Int = Bool\ntype Here = Int"), @r###"
    ---
    decls:
      - Type:
          name: Int
          params: []
          body: Bool
      - Type:
          name: Here
          params: []
          body:
            Syn: Int
    "###);
}

#[test]
fn type_con_syn_shadows_int() {
    insta::assert_yaml_snapshot!(check("type Int<A> = A\ntype Here = Int<Bool>"), @r###"
    ---
    decls:
      - Type:
          name: Int
          params:
            - A
          body:
            Var: A
      - Type:
          name: Here
          params: []
          body:
            App:
              - Syn: Int
              - - Bool
    "###);
}
