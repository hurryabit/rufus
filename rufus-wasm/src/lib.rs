use wasm_bindgen::prelude::*;

use rufus_core::humanizer;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct ExecResult {
    pub output: String,
    pub problems: String,
}

#[wasm_bindgen]
pub fn exec(program: &str) -> ExecResult {
    let _humanizer = humanizer::Humanizer::new(program);
    let parser = rufus_syntax::Parser::new(program);
    let result = parser.parse(rufus_syntax::rules::root);
    let output = rufus_syntax::dump_syntax(result.syntax, false);
    let mut problems = String::new();
    for error in result.errors {
        problems.push_str(&format!("{:?}\n", error));
    }
    ExecResult { output, problems }
}
