#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate lalrpop_util;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod cek;
mod syntax;
lalrpop_mod!(
    #[allow(clippy)]
    parser
);

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

#[test]
fn test() {
    use cek::*;
    let parser = parser::ExprParser::new();

    let expr1 = parser
        .parse(
            "
            let t = 3;
            let f = |x| (t * x);
            let twice = |f, x| f(f(x));
            twice(|x| twice(f, x), 2)",
        )
        .unwrap()
        .index()
        .unwrap();
    assert_eq!(State::init(&expr1).run().as_i64(), 162);

    let expr2 = parser
        .parse("let x = 1; let y = {let z = 2; z}; x")
        .unwrap()
        .index()
        .unwrap();
    assert_eq!(State::init(&expr2).run().as_i64(), 1);
}
