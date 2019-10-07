extern crate lalrpop_util;
extern crate rufus_core;
extern crate rustyline;

use rufus_core::{cek, parser, syntax};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use syntax::Expr;

const HISTORY_FILE: &str = ".rufus_history";

fn main() {
    println!("Hello!");
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history(HISTORY_FILE).is_err() {
        println!("No previous history.");
    }
    let parser = parser::ExprParser::new();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match parser
                    .parse(&line)
                    .map_err(|err| lalrpop_util::ParseError::to_string(&err))
                    .and_then(Expr::index)
                {
                    Ok(expr) => {
                        let state = cek::State::init(&expr);
                        let value = state.run();
                        println!("{:?}", value);
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => {
                println!("Good bye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(HISTORY_FILE).unwrap();
}

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
