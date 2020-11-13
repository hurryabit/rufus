use crate::*;
use syntax::Module;

fn check(input: &str) -> Module {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    module.check().unwrap();
    module
}

fn check_err(input: &str) -> String {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    let lerror = module.check().unwrap_err();
    format!("{}: {}", lerror.span, lerror.locatee)
}

#[test]
fn unknown_type_var() {
    insta::assert_snapshot!(check_err("type Bad = Unknown"), @"11-18: Undeclared type variable `Unknown`.");
}

#[test]
fn unexpected_type_con_at_top() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = Id"), @"26-28: Expected a type but found the generic type `Id`.");
}

#[test]
fn unexpected_type_con_in_type_args() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype List<A> = A\ntype Bad = List<Id>"), @"48-50: Expected a type but found the generic type `Id`.");
}

#[test]
fn unexpected_type_con_in_func_args() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = (Id) -> Int"), @"27-29: Expected a type but found the generic type `Id`.");
}

#[test]
fn unexpected_type_con_in_func_result() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = () -> Id"), @"32-34: Expected a type but found the generic type `Id`.");
}

#[test]
fn unexpected_type_con_in_record() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = {field: Id}"), @"34-36: Expected a type but found the generic type `Id`.");
}

#[test]
fn unexpected_type_con_in_variant() {
    insta::assert_snapshot!(check_err("type Id<A> = A\ntype Bad = [Constr(Id)]"), @"34-36: Expected a type but found the generic type `Id`.");
}

#[test]
fn wrong_arity_var() {
    insta::assert_snapshot!(check_err("type Bad<F> = F<Int>"), @"14-20: Type `F` is not a generic type but is applied to 1 type argument.");
}

#[test]
fn wrong_arity_builtin() {
    insta::assert_snapshot!(check_err("type Bad<A> = Int<A>"), @"14-20: Type `Int` is not a generic type but is applied to 1 type argument.");
}

#[test]
fn wrong_arity_type_syn() {
    insta::assert_snapshot!(check_err("type Syn = Int\ntype Bad = Syn<Int>"), @"26-34: Type `Syn` is not a generic type but is applied to 1 type argument.");
}

#[test]
fn wrong_arity_type_con_syn() {
    insta::assert_snapshot!(check_err("type Syn<A> = A\ntype Bad = Syn<Int, Int>"), @"27-40: Generic type `Syn` expects 1 type argument but is applied to 2 type arguments.");
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
