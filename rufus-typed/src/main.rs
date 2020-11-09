use rufus_typed::parser;

type ParseErrorRaw<'a> = lalrpop_util::ParseError<usize, parser::Token<'a>, &'static str>;
type ParseError = lalrpop_util::ParseError<usize, String, &'static str>;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Parse(ParseError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ParseErrorRaw<'_>> for Error {
    fn from(err: ParseErrorRaw) -> Self {
        Self::Parse(err.map_token(|t| format!("{}", t)))
    }

}

fn main() -> Result<(), Error> {
    let path = if let Some(path) = std::env::args().nth(1) {
        path
    } else {
        panic!("usage: {} <filename>", std::env::args().nth(0).unwrap())
    };
    let input = std::fs::read_to_string(path)?;
    let parser = parser::ModuleParser::new();
    let ast = parser.parse(&input)?;
    println!("{:#?}", ast);
    Ok(())
}
