use crate::*;
use syntax::{Decl, Module, Type, FuncDecl};

mod expressions;
mod resolution;
mod decls;
mod types;

#[allow(dead_code)]
fn check_output(input: &str) -> Module {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    module.check().unwrap();
    module
}

fn check_output_type(name: &str, input: &str) -> Type {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
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
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
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

fn check_success(input: &str) {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    if let Err(error) = module.check() {
        panic!(
            "Expected module to type check but got error\n{}: {}",
            error.span, error.locatee
        );
    }
}

fn check_error(input: &str) -> String {
    let parser = parser::ModuleParser::new();
    let mut errors = Vec::new();
    let mut module = parser.parse(&mut errors, input).unwrap();
    assert_eq!(errors, vec![]);
    let error = module.check().unwrap_err();
    let trans = util::PositionTranslator::new(input);
    let span = trans.span(error.span);
    let error = error.locatee;
    if span.start.line == span.end.line {
        let line = input.lines().nth(span.start.line).unwrap();
        format!(
            "{:3} | {}\n{}{}\n{}",
            span.start.line,
            line,
            " ".repeat(span.start.column + 6),
            "~".repeat(span.end.column - span.start.column),
            error
        )
    } else {
        format!("{}: {}", span, error)
    }
}
