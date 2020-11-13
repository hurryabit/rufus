use super::*;

#[test]
fn unknown_type_var() {
    insta::assert_snapshot!(check_err("type Bad = Unknown"), @r###"
      0 | type Bad = Unknown
                     ~~~~~~~
    Undeclared type variable `Unknown`.
    "###);
}

#[test]
fn unexpected_type_con_at_top() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = Id"), @r###"
      1 | type Bad = Id
                     ~~
    Expected a type but found the generic type `Id`.
    "###);
}

#[test]
fn unexpected_type_con_in_type_args() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype List<A> = A\ntype Bad = List<Id>"), @r###"
      2 | type Bad = List<Id>
                          ~~
    Expected a type but found the generic type `Id`.
    "###);
}

#[test]
fn unexpected_type_con_in_func_args() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = (Id) -> Int"), @r###"
      1 | type Bad = (Id) -> Int
                      ~~
    Expected a type but found the generic type `Id`.
    "###);
}

#[test]
fn unexpected_type_con_in_func_result() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = () -> Id"), @r###"
      1 | type Bad = () -> Id
                           ~~
    Expected a type but found the generic type `Id`.
    "###);
}

#[test]
fn unexpected_type_con_in_record() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = {field: Id}"), @r###"
      1 | type Bad = {field: Id}
                             ~~
    Expected a type but found the generic type `Id`.
    "###);
}

#[test]
fn unexpected_type_con_in_variant() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = [Constr(Id)]"), @r###"
      1 | type Bad = [Constr(Id)]
                             ~~
    Expected a type but found the generic type `Id`.
    "###);
}

#[test]
fn wrong_arity_var() {
    insta::assert_snapshot!(check_err("type Bad<F> = F<Int>"), @r###"
      0 | type Bad<F> = F<Int>
                        ~~~~~~
    Type `F` is not a generic type but is applied to 1 type argument.
    "###);
}

#[test]
fn wrong_arity_builtin() {
    insta::assert_snapshot!(check_err("type Bad<A> = Int<A>"), @r###"
      0 | type Bad<A> = Int<A>
                        ~~~~~~
    Type `Int` is not a generic type but is applied to 1 type argument.
    "###);
}

#[test]
fn wrong_arity_type_syn() {
    insta::assert_snapshot!(check_err("type Syn = Int\ntype Bad = Syn<Int>"), @r###"
      1 | type Bad = Syn<Int>
                     ~~~~~~~~
    Type `Syn` is not a generic type but is applied to 1 type argument.
    "###);
}

#[test]
fn wrong_arity_type_con_syn() {
    insta::assert_snapshot!(check_err("type Syn<A> = A\ntype Bad = Syn<Int, Int>"), @r###"
      1 | type Bad = Syn<Int, Int>
                     ~~~~~~~~~~~~~
    Generic type `Syn` expects 1 type argument but is applied to 2 type arguments.
    "###);
}

#[test]
fn int_resolved() {
    insta::assert_debug_snapshot!(check("type Here = Int"), @r###"
    MODULE
      decl: TYPEDECL
        name: Here @ 5...9
        type: INT @ 12...15
    "###);
}

#[test]
fn bool_resolved() {
    insta::assert_debug_snapshot!(check("type Here = Bool"), @r###"
    MODULE
      decl: TYPEDECL
        name: Here @ 5...9
        type: BOOL @ 12...16
    "###);
}

#[test]
fn syn_resolved() {
    insta::assert_debug_snapshot!(check("type Syn = Int\ntype Here = Syn"), @r###"
    MODULE
      decl: TYPEDECL
        name: Syn @ 5...8
        type: INT @ 11...14
      decl: TYPEDECL
        name: Here @ 20...24
        type: APP @ 27...30
          syn: Syn @ 27...30
    "###);
}

#[test]
fn var_resolved() {
    insta::assert_debug_snapshot!(check("type Here<A> = A"), @r###"
    MODULE
      decl: TYPEDECL
        name: Here @ 5...9
        type_param: A @ 10...11
        type: A @ 15...16
    "###);
}

#[test]
fn var_shadows_int() {
    insta::assert_debug_snapshot!(check("type Here<Int> = Int"), @r###"
    MODULE
      decl: TYPEDECL
        name: Here @ 5...9
        type_param: Int @ 10...13
        type: Int @ 17...20
    "###);
}

#[test]
fn type_syn_shadows_int() {
    insta::assert_debug_snapshot!(check("type Int = Bool\ntype Here = Int"), @r###"
    MODULE
      decl: TYPEDECL
        name: Int @ 5...8
        type: BOOL @ 11...15
      decl: TYPEDECL
        name: Here @ 21...25
        type: APP @ 28...31
          syn: Int @ 28...31
    "###);
}

#[test]
fn type_con_syn_shadows_int() {
    insta::assert_debug_snapshot!(check("type Int<A> = A\ntype Here = Int<Bool>"), @r###"
    MODULE
      decl: TYPEDECL
        name: Int @ 5...8
        type_param: A @ 9...10
        type: A @ 14...15
      decl: TYPEDECL
        name: Here @ 21...25
        type: APP @ 28...37
          syn: Int @ 28...31
          type_arg: BOOL @ 32...36
    "###);
}
