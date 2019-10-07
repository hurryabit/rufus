#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate lalrpop_util;

pub mod cek;
pub mod syntax;
lalrpop_mod!(
    #[allow(clippy)]
    pub parser
);

#[cfg(test)]
mod tests {
    fn integration_test(expected: i64, expr: &str) {
        use crate::cek::*;
        use crate::parser::ExprParser;
        let parser = ExprParser::new();
        let expr = parser.parse(expr).unwrap().index().unwrap();
        assert_eq!(State::init(&expr).run().as_i64(), expected);
    }

    #[test]
    fn twice() {
        integration_test(
            162,
            "
            let t = 3;
            let f = |x| (t * x);
            let twice = |f, x| f(f(x));
            twice(|x| twice(f, x), 2)",
        );
    }

    #[test]
    fn nested_let() {
        integration_test(1, "let x = 1; let y = {let z = 2; z}; x");
    }

    #[test]
    fn simple_lambda() {
        integration_test(5, "let x = 1; let y = 2; let f = |z| (x+z); f(4)");
    }
}
