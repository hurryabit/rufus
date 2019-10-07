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
                    .and_then(|expr| {
                        let machine = cek::Machine::new(&expr);
                        machine.run().map(|value| format!("{:?}", value))

                    })
                {
                    Ok(value) => println!("{}", value),
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
