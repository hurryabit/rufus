use crate::grammar;
use crate::syntax::Module;
use crate::location;
use lalrpop_util::ParseError;
use location::{Humanizer, ParserLoc, Span};
use crate::diagnostic::*;

impl Module {
    pub fn parse(input: &str, humanizer: &Humanizer) -> (Option<Self>, Vec<Diagnostic>) {
        let parser = grammar::ModuleParser::new();
        let mut errors = Vec::new();
        match parser.parse(&mut errors, &input) {
            Ok(module) => {
                let diagnostics = errors
                    .into_iter()
                    .map(|error| parse_error_to_diagnostic(error, humanizer))
                    .collect::<Vec<_>>();
                (Some(module), diagnostics)
            }
            Err(fatal_error) => {
                let error = errors
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| fatal_error.map_location(ParserLoc::from_usize));
                let diagnostics = vec![parse_error_to_diagnostic(error, humanizer)];
                (None, diagnostics)
            }
        }
    }
}

pub fn parse_error_to_diagnostic(
    error: ParseError<ParserLoc, grammar::Token<'_>, &'static str>,
    humanizer: &Humanizer,
) -> Diagnostic {
    use ParseError::*;
    let error = error.map_location(|loc| loc.humanize(humanizer));
    let span = match error {
        InvalidToken { location } | UnrecognizedEOF { location, .. } => Span::new(location, location),
        UnrecognizedToken {
            token: (start, _, end),
            ..
        }
        | ExtraToken {
            token: (start, _, end),
        } => Span::new(start, end),
        User { .. } => Span::default(),
    };
    Diagnostic {
        span,
        severity: Severity::Error,
        source: Source::Parser,
        message: format!("{}", error),
    }
}
