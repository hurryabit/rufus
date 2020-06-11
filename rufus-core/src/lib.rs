#![allow(renamed_and_removed_lints)]
#![allow(unused_parens)]
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
        assert_eq!(
            Machine::new(&expr).run().unwrap().as_i64().unwrap(),
            expected
        );
    }

    #[test]
    fn twice() {
        integration_test(
            162,
            "
            let t = 3 in
            let f = fun x -> t * x in
            let twice = fun f x -> f (f x) in
            twice (twice f) 2
            ",
        );
    }

    #[test]
    fn nested_let() {
        integration_test(
            1,
            "
            let x = 1 in
            let y =
                let z = 2 in
                z
            in
            x
            ",
        );
    }

    #[test]
    fn simple_lambda() {
        integration_test(
            5,
            "
            let x = 1 in
            let y = 2 in
            let f = fun z -> x + z in
            f 4
            ",
        );
    }

    #[test]
    fn simple_record() {
        integration_test(
            1,
            "
            let pair = fun x y -> { x = x; y = y } in
            let fst = fun p -> p.x in
            fst (pair 1 2)
            ",
        );
    }

    // #[test]
    // fn simple_variant() {
    //     integration_test(
    //         1,
    //         "
    //         let ok = Ok(1);
    //         let x = 2;
    //         match ok {
    //             Ok(x) => x,
    //             Err(e) => e
    //         }",
    //     );
    // }
}
