use rufus_typed::parser;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Parse(lalrpop_util::ParseError<usize, String, &'static str>),
    Yaml(serde_yaml::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<lalrpop_util::ParseError<usize, parser::Token<'_>, &'static str>> for Error {
    fn from(err: lalrpop_util::ParseError<usize, parser::Token<'_>, &'static str>) -> Self {
        Self::Parse(err.map_token(|t| format!("{}", t)))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err)
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
    serde_yaml::to_writer(std::io::stdout(), &ast)?;
    Ok(())
}
