use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

const HISTORY_FILE: &str = ".rufus_history";

fn print_syntax(root: rufus_syntax::SyntaxNode) {
    fn go(node: rufus_syntax::SyntaxNode, indent: &mut String) {
        println!("{}{:?}", indent, node);
        indent.push_str("  ");
        for child in node.children_with_tokens() {
            match child {
                rufus_syntax::SyntaxElement::Node(node) => go(node, indent),
                rufus_syntax::SyntaxElement::Token(token) => {
                    if !token.kind().is_trivia() {
                        println!("{}#{:?}", indent, token);
                    }
                }
            }
        }
        indent.truncate(indent.len() - 2);
    }

    println!("// Nodes prefixed with `#` are tokens.");
    go(root, &mut String::new());
}

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
                let result = parser.parse();
                for error in result.errors {
                    println!("ERROR: {:?}", error);
                }
                print_syntax(result.syntax);
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
