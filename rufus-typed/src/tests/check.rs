use crate::*;
use syntax::Module;

mod expressions;
mod signatures;
mod types;

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
