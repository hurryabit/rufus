use super::*;

#[test]
fn unknown_type_var_in_let() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { let x: A = 0; x }
    "#), @r###"
      1 |     fn f() -> Int { let x: A = 0; x }
                                     ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_inferrable_lambda() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { fn (x: A) { 0 } }
    "#), @r###"
      1 |     fn f() -> Int { fn (x: A) { 0 } }
                                     ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_checkable_lambda() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { fn (x: A, y) { 0 } }
    "#), @r###"
      1 |     fn f() -> Int { fn (x: A, y) { 0 } }
                                     ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_func_inst() {
    insta::assert_snapshot!(check_err(r#"
    fn g<A>(x: A) -> A { x }
    fn f() -> Int { g@<A>() }
    "#), @r###"
      2 |     fn f() -> Int { g@<A>() }
                                 ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn rule_check_infer() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { true }
    "#), @r###"
      1 |     fn f() -> Int { true }
                              ~~~~
    Expected an expression of type `Int` but found an expression of type `Bool`.
    "###);
}

#[test]
fn rule_ann() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { let x: Int = true; 0 }
    "#), @r###"
      1 |     fn f() -> Int { let x: Int = true; 0 }
                                           ~~~~
    Expected an expression of type `Int` but found an expression of type `Bool`.
    "###);
}

#[test]
fn rule_no_ann() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { let x = true; x }
    "#), @r###"
      1 |     fn f() -> Int { let x = true; x }
                                            ~
    Expected an expression of type `Int` but found an expression of type `Bool`.
    "###);
}

#[test]
fn rule_var() {
    insta::assert_snapshot!(check_err(r#"
    fn f(x: Bool) -> Int { x }
    "#), @r###"
      1 |     fn f(x: Bool) -> Int { x }
                                     ~
    Expected an expression of type `Int` but found an expression of type `Bool`.
    "###);
}

#[test]
fn rule_lit_int() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Bool { 0 }
    "#), @r###"
      1 |     fn f() -> Bool { 0 }
                               ~
    Expected an expression of type `Bool` but found an expression of type `Int`.
    "###);
}

#[test]
fn rule_lit_bool_true() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { true }
    "#), @r###"
      1 |     fn f() -> Int { true }
                              ~~~~
    Expected an expression of type `Int` but found an expression of type `Bool`.
    "###);
}

#[test]
fn rule_lit_bool_false() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { false }
    "#), @r###"
      1 |     fn f() -> Int { false }
                              ~~~~~
    Expected an expression of type `Int` but found an expression of type `Bool`.
    "###);
}

#[test]
fn rule_lam_infer() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { let f = fn (x: Int) { x }; f }
    "#), @r###"
      1 |     fn f() -> Int { let f = fn (x: Int) { x }; f }
                                                         ~
    Expected an expression of type `Int` but found an expression of type `(Int) -> Int`.
    "###);
}

#[test]
fn rule_lam_check_no_func() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> Int { fn () { 0 } }
    "#), @r###"
      1 |     fn f() -> Int { fn () { 0 } }
                              ~~~~~~~~~~~
    Expected an expression of type `Int` but found an expression of type `() -> Int`.
    "###);
}

#[test]
fn rule_lam_check_bad_arity() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> () -> Int { fn (x) { 0 } }
    "#), @r###"
      1 |     fn f() -> () -> Int { fn (x) { 0 } }
                                    ~~~~~~~~~~~~
    Expected an expression of type `() -> Int` but found a lambda with 1 parameter.
    "###);
}

#[test]
fn rule_lam_check_bad_param() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> (Int, Int) -> Int { fn (x, y: Bool) { 0 } }
    "#), @r###"
      1 |     fn f() -> (Int, Int) -> Int { fn (x, y: Bool) { 0 } }
                                                      ~~~~
    Expected parameter `y` to have type `Int` but found a type annotation `Bool`.
    "###);
}

#[test]
fn rule_lam_check_bad_result() {
    insta::assert_snapshot!(check_err(r#"
    fn f() -> (Int, Int) -> Bool { fn (x, y: Int) { x + y } }
    "#), @r###"
      1 |     fn f() -> (Int, Int) -> Bool { fn (x, y: Int) { x + y } }
                                                              ~~~~~
    Expected an expression of type `Bool` but found an expression of type `Int`.
    "###);
}
