use crate::*;
use syntax::{Decl, Expr, Module, Type, FuncDecl};

mod expressions;
mod decls;
mod func_resolution;
mod shadowing;
mod type_resolution;
mod types;

#[allow(dead_code)]
fn check_output(input: &str) -> Module {
    let (result, diagnostics) = Module::parse_test(input);
    assert!(diagnostics.is_empty());
    let mut module = result.unwrap();
    module.check().unwrap();
    module
}

fn check_output_type(name: &str, input: &str) -> Type {
    let (result, diagnostics) = Module::parse_test(input);
    assert!(diagnostics.is_empty());
    let mut module = result.unwrap();
    module.check().unwrap();
    module
        .decls
        .into_iter()
        .find_map(|decl| match decl {
            Decl::Type(decl) if decl.name.locatee.with_name(|n| n == name) => {
                Some(decl.body.locatee)
            }
            _ => None,
        })
        .unwrap()
}

fn check_output_func_decl(name: &str, input: &str) -> FuncDecl {
    let (result, diagnostics) = Module::parse_test(input);
    assert!(diagnostics.is_empty());
    let mut module = result.unwrap();
    module.check().unwrap();
    module
        .decls
        .into_iter()
        .find_map(|decl| match decl {
            Decl::Func(decl) if decl.name.locatee.with_name(|n| n == name) => {
                Some(decl)
            }
            _ => None,
        })
        .unwrap()
}

fn check_output_func_body(name: &str, input: &str) -> Expr {
    check_output_func_decl(name, input).body.locatee
}


fn check_success(input: &str) {
    let (result, diagnostics) = Module::parse_test(input);
    assert!(diagnostics.is_empty());
    let mut module = result.unwrap();
    if let Err(error) = module.check() {
        panic!(
            "Expected module to type check but got error\n{:?}: {}",
            error.span, error.locatee
        );
    }
}

fn check_error(input: &str) -> String {
    let humanizer = location::Humanizer::new(input);
    let (result, diagnostics) = Module::parse(input, &humanizer);
    assert!(diagnostics.is_empty());
    let mut module = result.unwrap();
    let error = module.check().unwrap_err();
    let span = error.span.humanize(&humanizer);
    let error = error.locatee;
    if span.start.line == span.end.line {
        let line = input.lines().nth(span.start.line as usize).unwrap();
        format!(
            "{:3} | {}\n{}{}\n{}",
            span.start.line,
            line,
            " ".repeat((span.start.column + 6) as usize),
            "~".repeat((span.end.column - span.start.column) as usize),
            error
        )
    } else {
        format!("{}: {}", span, error)
    }
}
