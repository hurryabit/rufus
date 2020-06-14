#[macro_use]
extern crate lalrpop_util;

pub mod cek;
pub mod syntax;
lalrpop_mod!(
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

    fn example_test(expected: i64, path: &str) {
        use std::io::Read;
        let mut file = std::fs::File::open(path).unwrap();
        let mut expr = String::new();
        file.read_to_string(&mut expr).unwrap();
        integration_test(expected, &expr)
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

    #[test]
    fn fib() {
        example_test(55, "../examples/fib.ml")
    }

    #[test]
    fn list() {
        example_test(55, "../examples/list.ml")
    }
}
