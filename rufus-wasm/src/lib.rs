use wasm_bindgen::prelude::*;

use rufus_core::humanizer::Humanizer;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Problem {
    pub start: Location,
    pub end: Location,
    // TODO(MH): Figure out a way to make this an enum (with string value on the JS side).
    pub severity: String,
    pub message: String,
    pub source: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct ExecResult {
    pub output: String,
    pub problems: Vec<Problem>,
}

impl Location {
    fn from_index(index: u32, humanizer: &Humanizer) -> Self {
        let loc = humanizer.run(index as usize);
        Self {
            line: loc.line,
            column: loc.column,
        }
    }
}

impl Problem {
    fn from_parse_error(error: rufus_syntax::ParseError, humanizer: &Humanizer) -> Self {
        let span = error.span;
        Self {
            start: Location::from_index(span.start, humanizer),
            end: Location::from_index(span.end, humanizer),
            severity: String::from("ERROR"),
            message: format!("found {:?}, expected {:?}", error.found, error.expected),
            source: format!("parser ({})", error.rule),
        }
    }
}

#[wasm_bindgen]
pub fn exec(program: &str) -> ExecResult {
    let humanizer = Humanizer::new(program);
    let parser = rufus_syntax::Parser::new(program);
    let result = parser.parse(rufus_syntax::rules::root);
    let output = rufus_syntax::dump_syntax(result.syntax, false);
    let problems: Vec<_> = result
        .errors
        .into_iter()
        .map(|error| Problem::from_parse_error(error, &humanizer))
        .collect();
    ExecResult { output, problems }
}
