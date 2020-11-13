use super::*;

#[test]
fn unknown_type_var_in_let() {
    insta::assert_snapshot!(check_err("fn f() -> Int { let x: A = 0; x }"), @r###"
      0 | fn f() -> Int { let x: A = 0; x }
                                 ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_inferrable_lambda() {
    insta::assert_snapshot!(check_err("fn f() -> Int { fn (x: A) { 0 } }"), @r###"
      0 | fn f() -> Int { fn (x: A) { 0 } }
                                 ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_checkable_lambda() {
    insta::assert_snapshot!(check_err("fn f() -> Int { fn (x: A, y) { 0 } }"), @r###"
      0 | fn f() -> Int { fn (x: A, y) { 0 } }
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
