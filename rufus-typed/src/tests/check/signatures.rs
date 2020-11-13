use super::*;

#[test]
fn duplicate_type_var() {
    insta::assert_snapshot!(check_err("fn f<A, A>() -> A { 0 }"), @r###"
      0 | fn f<A, A>() -> A { 0 }
                  ~
    Duplicate type variable `A`.
    "###);
}

#[test]
fn unknown_type_var() {
    insta::assert_snapshot!(check_err("fn f() -> A { 0 }"), @r###"
      0 | fn f() -> A { 0 }
                    ~
    Undeclared type variable `A`.
    "###);
}

#[test]
fn kind_error_in_param() {
    insta::assert_snapshot!(check_err("fn f<A>(x: A<Int>) -> A { 0 }"), @r###"
      0 | fn f<A>(x: A<Int>) -> A { 0 }
                     ~~~~~~
    Type `A` is not a generic type but is applied to 1 type argument.
    "###);
}

#[test]
fn kind_error_in_result() {
    insta::assert_snapshot!(check_err("fn map<A>(x: A) -> A<Int> { 0 }"), @r###"
      0 | fn map<A>(x: A) -> A<Int> { 0 }
                             ~~~~~~
    Type `A` is not a generic type but is applied to 1 type argument.
    "###);
}
