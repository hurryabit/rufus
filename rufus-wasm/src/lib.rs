use wasm_bindgen::prelude::*;

use rufus_core::{cek, humanizer, parser};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum ExecResultStatus {
    Ok,
    Err,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct ExecResult {
    pub status: ExecResultStatus,
    value: String,
}

#[wasm_bindgen]
impl ExecResult {
    pub fn get_value(self) -> String {
        self.value
    }
}

fn exec_result(program: &str) -> Result<String, String> {
    let humanizer = humanizer::Humanizer::new(program);
    let parser = parser::ExprParser::new();
    let expr = parser
        .parse(program)
        .map_err(|err| {
            let mut msg = err.map_location(|loc| humanizer.run(loc)).to_string();
            humanizer::sanitize_source_span(&mut msg);
            msg
        })?
        .index()?;
    let machine = cek::Machine::new(&expr);
    let value = machine.run()?;
    Ok(value.to_string())
}

#[wasm_bindgen]
pub fn exec(program: &str) -> ExecResult {
    match exec_result(program) {
        Ok(value) => ExecResult {
            status: ExecResultStatus::Ok,
            value,
        },
        Err(msg) => ExecResult {
            status: ExecResultStatus::Err,
            value: msg,
        },
    }
}
