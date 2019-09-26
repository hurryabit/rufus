#[macro_use]
extern crate lalrpop_util;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod eval;
mod syntax;
lalrpop_mod!(parser);

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
                    Ok(expr) => println!("{}", expr.eval()),
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
    assert_eq!(parser.parse("1+2*3").unwrap().eval(), 7);
    assert!(parser.parse("a").is_err());
}
