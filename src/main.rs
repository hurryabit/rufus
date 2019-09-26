#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate lalrpop_util;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod eval;
mod syntax;
lalrpop_mod!(
    #[allow(clippy)]
    parser
);

use eval::Env;

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
                match parser.parse(&line) {
                    Ok(expr) => match expr.eval(Env::new()) {
                        Ok(val) => println!("{}", val.as_i64().unwrap()),
                        Err(err) => println!("Error: {}", err),
                    },
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
fn parser() {
    let parser = parser::ExprParser::new();
    assert_eq!(parser.parse("let t = 3; let f = |x| { *(t, x) }; let twice = |f, x| { f(f(x)) }; twice(|x| { twice(f, x) }, 2)").unwrap().eval(Env::new()).unwrap().as_i64().unwrap(), 162);
}
