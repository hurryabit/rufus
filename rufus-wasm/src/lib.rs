extern crate lalrpop_util;
extern crate rufus_core;

use wasm_bindgen::prelude::*;

use rufus_core::{cek, parser, syntax};
use syntax::Expr;

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

#[wasm_bindgen]
pub fn exec(program: &str) -> ExecResult {
    let parser = parser::ExprParser::new();
    match parser
        .parse(program)
        .map_err(|err| lalrpop_util::ParseError::to_string(&err))
        .and_then(Expr::index)
    {
        Ok(expr) => {
            let state = cek::State::init(&expr);
            let value = state.run();
            let value = format!("{:?}", value);
            ExecResult {
                status: ExecResultStatus::Ok,
                value: value
            }
        }
        Err(msg) => ExecResult {
            status: ExecResultStatus::Err,
            value: msg,
        }
    }
}
