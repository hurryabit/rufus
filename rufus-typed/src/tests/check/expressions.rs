/*
Some notes:

* To check that the of an expression `E` can be inferred and matches `T`, we
  structure the test as
  ```
  fn () -> T {
      let x = E;
      x
  }
  ```
  The unannotated `let`-binding ensure that the type of `E` gets definitely
  inferred rather than (accidentally) checked against `T`.

* If we want to check that type inference fails for an expression, we use a
  variant constructor like `InferMe` to signal that.

* If we want to test that the type of an expression gets checked and fails, we
  use a variant constructor `CheckMe` to signal that.
*/

use super::*;

#[test]
fn unknown_type_var_in_let() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x: A = 0;
        x
    }
    "#), @r###"
      2 |         let x: A = 0;
                         ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_inferrable_lambda() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        fn (x: A) { 0 }
    }
    "#), @r###"
      2 |         fn (x: A) { 0 }
                         ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_checkable_lambda() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        fn (x: A, y) { 0 }
    }
    "#), @r###"
      2 |         fn (x: A, y) { 0 }
                         ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn unknown_type_var_in_func_inst() {
    insta::assert_snapshot!(check_error(r#"
    fn g<A>(x: A) -> A { x }
    fn f() -> Int {
        g@<A>()
    }
    "#), @r###"
      3 |         g@<A>()
                     ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn rule_check_infer_good() {
    check_success(r#"
    fn f() -> Int { 0 }
    "#);
}

#[test]
fn rule_check_infer_mismatch() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Bool { 0 }
    "#), @r###"
      1 |     fn f() -> Bool { 0 }
                               ~
    Expected an expression of type `Bool` but found an expression of type `Int`.
    "###);
}

#[test]
fn rule_var() {
    check_success(r#"
    fn f(x: Int) -> Int {
        let y = x;
        y
    }
    "#);
}

#[test]
fn rule_lit_int_0() {
    check_success(r#"
    fn f() -> Int {
        let x = 0;
        x
    }
    "#);
}

#[test]
fn rule_lit_int_1() {
    check_success(r#"
    fn f() -> Int {
        let x = 1;
        x
    }
    "#);
}

#[test]
fn rule_lit_bool_true() {
    check_success(r#"
    fn f() -> Bool {
        let x = true;
        x
    }
    "#);
}

#[test]
fn rule_lit_bool_false() {
    check_success(r#"
    fn f() -> Bool {
        let x = false;
        x
    }
    "#);
}

#[test]
fn rule_lam_infer_0() {
    check_success(r#"
    fn f() -> () -> Int {
        let f = fn () { 1 };
        f
    }
    "#);
}

#[test]
fn rule_lam_infer_1() {
    check_success(r#"
    fn f() -> (Int) -> Int {
        let f = fn (x: Int) { x };
        f
    }
    "#);
}

#[test]
fn rule_lam_infer_duplicate_param() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int) -> Bool {
        let f = fn (x: Int, x: Int) { x };
        f
    }
    "#), @r###"
      2 |         let f = fn (x: Int, x: Int) { x };
                                      ~
    Duplicate paramter `x`.
    "###);
}

#[test]
fn rule_lam_infer_not_inferrable() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int) -> Bool {
        let f = fn (x) {
            let y: Int = x;
            y
        };
        f
    }
    "#), @r###"
      2 |         let f = fn (x) {
                              ~
    Cannot infer the type of parameter `x`. A type annoation is needed.
    "###);
}

#[test]
fn rule_lam_check_0() {
    check_success(r#"
    fn f() -> () -> Int {
        fn () { 1 }
    }
    "#);
}

#[test]
fn rule_lam_check_1() {
    check_success(r#"
    fn f() -> (Int) -> Int {
        fn (x) { x }
    }
    "#);
}

#[test]
fn rule_lam_check_no_func() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        fn (x) { 0 }
    }
    "#), @r###"
      2 |         fn (x) { 0 }
                  ~~~~~~~~~~~~
    Expected an expression of type `Int` but found a lambda with 1 parameter.
    "###);
}

#[test]
fn rule_lam_check_duplicate_param_annotated() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int, Int) -> Int {
        fn (x: Int, x: Int) { 0 }
    }
    "#), @r###"
      2 |         fn (x: Int, x: Int) { 0 }
                              ~
    Duplicate paramter `x`.
    "###);
}

#[test]
fn rule_lam_check_duplicate_param_not_annotated() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int, Int) -> Int {
        fn (x, x) { 0 }
    }
    "#), @r###"
      2 |         fn (x, x) { 0 }
                         ~
    Duplicate paramter `x`.
    "###);
}

#[test]
fn rule_lam_check_too_many_params() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> () -> Int {
        fn (x) { 0 }
    }
    "#), @r###"
      2 |         fn (x) { 0 }
                  ~~~~~~~~~~~~
    Expected an expression of type `() -> Int` but found a lambda with 1 parameter.
    "###);
}

#[test]
fn rule_lam_check_too_few_params() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int) -> Int {
        fn () { 0 }
    }
    "#), @r###"
      2 |         fn () { 0 }
                  ~~~~~~~~~~~
    Expected an expression of type `(Int) -> Int` but found a lambda with 0 parameters.
    "###);
}

#[test]
fn rule_lam_check_mismatch_param_1() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int) -> Int {
        fn (x: Bool) { 0 }
    }
    "#), @r###"
      2 |         fn (x: Bool) { 0 }
                         ~~~~
    Expected parameter `x` to have type `Int` but found a type annotation `Bool`.
    "###);
}

#[test]
fn rule_lam_check_mismatch_param_2() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int, Int) -> Int {
        fn (x, y: Bool) { 0 }
    }
    "#), @r###"
      2 |         fn (x, y: Bool) { 0 }
                            ~~~~
    Expected parameter `y` to have type `Int` but found a type annotation `Bool`.
    "###);
}

#[test]
fn rule_lam_check_mismatch_result() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> (Int, Int) -> Bool {
        fn (x, y: Int) { CheckMe }
    }
    "#), @r###"
      2 |         fn (x, y: Int) { CheckMe }
                                   ~~~~~~~
    Expected an expression of type `Bool` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_func_inst_1() {
    check_success(r#"
    fn g<A>(x: A) -> A { x }
    fn f() -> Int {
        let x = g@<Int>(0);
        x
    }
    "#);
}

#[test]
fn rule_func_inst_on_var() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let g = fn () { 0 };
        g@<Int>()
    }
    "#), @r###"
      3 |         g@<Int>()
                  ~~~~~~~
    `g` is not a generic function but is applied to 1 type argument.
    "###);
}

#[test]
fn rule_func_inst_on_mono_func() {
    insta::assert_snapshot!(check_error(r#"
    fn g() -> Int { 0 }
    fn f() -> Int {
        g@<Int>()
    }
    "#), @r###"
      3 |         g@<Int>()
                  ~~~~~~~
    `g` is not a generic function but is applied to 1 type argument.
    "###);
}

#[test]
fn rule_func_inst_no_types_on_poly_func() {
    insta::assert_snapshot!(check_error(r#"
    fn g<A>(x: A) -> A { x }
    fn f() -> Int {
        g(1)
    }
    "#), @r###"
      3 |         g(1)
                  ~
    Generic function `g` expects 1 type argument but is applied to 0 type arguments.
    "###);
}

#[test]
fn rule_func_inst_too_many_types() {
    insta::assert_snapshot!(check_error(r#"
    fn g<A>(x: A) -> A { x }
    fn f() -> Int {
        g@<Int, Bool>(1)
    }
    "#), @r###"
      3 |         g@<Int, Bool>(1)
                  ~~~~~~~~~~~~~
    Generic function `g` expects 1 type argument but is applied to 2 type arguments.
    "###);
}

#[test]
fn rule_func_inst_too_few_types() {
    insta::assert_snapshot!(check_error(r#"
    fn g<A, B>(x: A, y: B) -> A { x }
    fn f() -> Int {
        g@<Int>(1)
    }
    "#), @r###"
      3 |         g@<Int>(1)
                  ~~~~~~~
    Generic function `g` expects 2 type arguments but is applied to 1 type argument.
    "###);
}

#[test]
fn rule_func_inst_mismatch_param() {
    insta::assert_snapshot!(check_error(r#"
    fn g<A>(x: A) -> Int { 0 }
    fn f() -> Int {
        g@<Int>(true)
    }
    "#), @r###"
      3 |         g@<Int>(true)
                          ~~~~
    Expected an expression of type `Int` but found an expression of type `Bool`.
    "###);
}

#[test]
fn rule_func_inst_mismatch_result() {
    insta::assert_snapshot!(check_error(r#"
    fn g<A>(x: A) -> A { x }
    fn f() -> Int {
        g@<Bool>(CheckMe)
    }
    "#), @r###"
      3 |         g@<Bool>(CheckMe)
                           ~~~~~~~
    Expected an expression of type `Bool` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_app_func_0() {
    check_success(r#"
    fn f() -> Int { 0 }
    fn g() -> Int {
        let x = f();
        x
    }
    "#);
}

#[test]
fn rule_app_func_1() {
    check_success(r#"
    fn f(x: Int) -> Int { x }
    fn g() -> Int {
        let x = f(1);
        x
    }
    "#);
}

#[test]
fn rule_app_func_2() {
    check_success(r#"
    fn f(x: Int, y: Int) -> Int { x + y }
    fn g() -> Int {
        let x = f(1, 2);
        x
    }
    "#);
}

#[test]
fn rule_app_func_poly() {
    check_success(r#"
    fn f<A>(x: A) -> A { x }
    fn g() -> Int {
        let x = f@<Int>(1);
        x
    }
    "#);
}

#[test]
fn rule_app_var() {
    check_success(r#"
    fn g() -> Int {
        let f = fn (x: Int) { x };
        let x = f(1);
        x
    }
    "#);
}


#[test]
fn rule_app_var_no_func() {
    insta::assert_snapshot!(check_error(r#"
    fn f(x: Int) -> Int {
        x()
    }
    "#), @r###"
      2 |         x()
                  ~~~
    `x` cannot be applied to 0 arguments because it has has type `Int`.
    "###);
}

#[test]
fn rule_app_var_too_many_args() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let g = fn () { 0 };
        g(1)
    }
    "#), @r###"
      3 |         g(1)
                  ~~~~
    `g` cannot be applied to 1 argument because it has has type `() -> Int`.
    "###);
}

#[test]
fn rule_app_var_too_few_args() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let g = fn (x: Int) { x };
        g()
    }
    "#), @r###"
      3 |         g()
                  ~~~
    `g` cannot be applied to 0 arguments because it has has type `(Int) -> Int`.
    "###);
}

#[test]
fn rule_app_func_too_many_args() {
    insta::assert_snapshot!(check_error(r#"
    fn g() -> Int { 0 }
    fn f() -> Int {
        g(1)
    }
    "#), @r###"
      3 |         g(1)
                  ~~~~
    `g` cannot be applied to 1 argument because it has has type `() -> Int`.
    "###);
}

#[test]
fn rule_app_func_too_few_args() {
    insta::assert_snapshot!(check_error(r#"
    fn g(x: Int) -> Int { x }
    fn f() -> Int {
        g()
    }
    "#), @r###"
      3 |         g()
                  ~~~
    `g` cannot be applied to 0 arguments because it has has type `(Int) -> Int`.
    "###);
}

#[test]
fn rule_app_func_mismatch_arg1() {
    insta::assert_snapshot!(check_error(r#"
    fn g(x: Int) -> Int { x }
    fn f() -> Int {
        g(CheckMe)
    }
    "#), @r###"
      3 |         g(CheckMe)
                    ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_app_var_mismatch_arg2() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let g = fn (x: Int, y: Bool) { x };
        g(1, CheckMe)
    }
    "#), @r###"
      3 |         g(1, CheckMe)
                       ~~~~~~~
    Expected an expression of type `Bool` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_binop_arith() {
    check_success(r#"
    fn f() -> Int {
        let x = 1 + 1;
        x
    }
    "#);
}

#[test]
fn rule_binop_arith_mismatch_lhs() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        CheckMe - 0
    }
    "#), @r###"
      2 |         CheckMe - 0
                  ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_binop_arith_mismatch_rhs() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        0 * CheckMe
    }
    "#), @r###"
      2 |         0 * CheckMe
                      ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_binop_cmp() {
    check_success(r#"
    fn f() -> Bool {
        let x = 1 == 1;
        x
    }
    "#);
}

#[test]
fn rule_binop_cmp_lhs_not_inferrable() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Bool {
        InferMe < CheckMe
    }
    "#), @r###"
      2 |         InferMe < CheckMe
                  ~~~~~~~
    Cannot infer the type of the expression. Further type annotations are required.
    "###);
}

#[test]
fn rule_binop_cmp_mismatch() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Bool {
        0 >= CheckMe
    }
    "#), @r###"
      2 |         0 >= CheckMe
                       ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_let_infer_infer() {
    check_success(r#"
    fn f() -> Int {
        let x = {
            let y = 1;
            y
        };
        x
    }
    "#);
}

#[test]
fn rule_let_infer_infer_bindee_not_inferrable() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = {
            let y = InferMe;
            0
        };
        0
    }
    "#), @r###"
      3 |             let y = InferMe;
                              ~~~~~~~
    Cannot infer the type of the expression. Further type annotations are required.
    "###);
}

#[test]
fn rule_let_infer_infer_body_not_inferrable() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = {
            let y = 0;
            InferMe
        };
        0
    }
    "#), @r###"
      4 |             InferMe
                      ~~~~~~~
    Cannot infer the type of the expression. Further type annotations are required.
    "###);
}

#[test]
fn rule_let_check_infer() {
    check_success(r#"
    fn f() -> Int {
        let x = {
            let y: [CheckMe] = CheckMe;
            0
        };
        x
    }
    "#);
}

#[test]
fn rule_let_check_infer_mismatch_bindee() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = {
            let y: Int = CheckMe;
            0
        };
        0
    }
    "#), @r###"
      3 |             let y: Int = CheckMe;
                                   ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_let_check_infer_body_not_inferrable() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = {
            let y: Int = 0;
            InferMe
        };
        0
    }
    "#), @r###"
      4 |             InferMe
                      ~~~~~~~
    Cannot infer the type of the expression. Further type annotations are required.
    "###);
}

#[test]
fn rule_let_infer_check() {
    check_success(r#"
    fn f() -> [CheckMe] {
        let x = 0;
        CheckMe
    }
    "#);
}

#[test]
fn rule_let_infer_check_bindee_not_inferrable() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = InferMe;
        0
    }
    "#), @r###"
      2 |         let x = InferMe;
                          ~~~~~~~
    Cannot infer the type of the expression. Further type annotations are required.
    "###);
}

#[test]
fn rule_let_infer_check_mismatch_body() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = 0;
        CheckMe
    }
    "#), @r###"
      3 |         CheckMe
                  ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_let_check_check() {
    check_success(r#"
    fn f() -> [CheckMe1] {
        let x: [CheckMe2] = CheckMe2;
        CheckMe1
    }
    "#);
}

#[test]
fn rule_let_check_check_mismatch_bindee() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> [CheckMe1] {
        let x: Int = CheckMe2;
        CheckMe1
    }
    "#), @r###"
      2 |         let x: Int = CheckMe2;
                               ~~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe2`.
    "###);
}

#[test]
fn rule_let_check_check_mismatch_body() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x: [CheckMe2] = CheckMe2;
        CheckMe1
    }
    "#), @r###"
      3 |         CheckMe1
                  ~~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe1`.
    "###);
}

#[test]
fn rule_if_infer() {
    check_success(r#"
    fn check_me() -> [CheckMe] { CheckMe }
    fn f() -> [CheckMe] {
        let x = if true {
            check_me()
        } else {
            CheckMe
        };
        x
    }
    "#);
}

#[test]
fn rule_if_infer_cond_mismatch() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = if CheckMe { 1 } else { 2 };
        0
    }
    "#), @r###"
      2 |         let x = if CheckMe { 1 } else { 2 };
                             ~~~~~~~
    Expected an expression of type `Bool` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_if_infer_then_not_inferrable() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = if true { InferMe } else { 1 };
        0
    }
    "#), @r###"
      2 |         let x = if true { InferMe } else { 1 };
                                    ~~~~~~~
    Cannot infer the type of the expression. Further type annotations are required.
    "###);
}

#[test]
fn rule_if_infer_else_mismatch() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        let x = if true { 0 } else { CheckMe };
        x
    }
    "#), @r###"
      2 |         let x = if true { 0 } else { CheckMe };
                                               ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_if_check() {
    check_success(r#"
    fn f() -> [CheckMe] {
        if true {
            CheckMe
        } else {
            CheckMe
        }
    }
    "#);
}

#[test]
fn rule_if_check_cond_mismatch() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        if CheckMe { 1 } else { 2 }
    }
    "#), @r###"
      2 |         if CheckMe { 1 } else { 2 }
                     ~~~~~~~
    Expected an expression of type `Bool` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_if_check_then_bad() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        if true { CheckMe } else { 1 }
    }
    "#), @r###"
      2 |         if true { CheckMe } else { 1 }
                            ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}

#[test]
fn rule_if_check_else_bad() {
    insta::assert_snapshot!(check_error(r#"
    fn f() -> Int {
        if true { 0 } else { CheckMe }
    }
    "#), @r###"
      2 |         if true { 0 } else { CheckMe }
                                       ~~~~~~~
    Expected an expression of type `Int` but found variant constructor `CheckMe`.
    "###);
}
