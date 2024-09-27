use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

const HISTORY_FILE: &str = ".rufus_history";

fn main() {
    println!("Hello!");
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new().expect("Cannot create readline editor.");
    if rl.load_history(HISTORY_FILE).is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let parser = rufus_syntax::Parser::new(&line);
                let result = parser.parse(rufus_syntax::rules::root);
                for error in result.errors {
                    println!("ERROR: {:?}", error);
                }
                println!("// Nodes prefixed with `#` are tokens.");
                print!("{}", rufus_syntax::dump_syntax(result.syntax, false));
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
