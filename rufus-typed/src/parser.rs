use crate::grammar;
use crate::syntax::Module;
use crate::util;
use lalrpop_util::ParseError;
use lsp_types::{Diagnostic, DiagnosticSeverity, Range};
use util::PositionTranslator;

impl Module {
    pub fn parse(input: &str, translator: &PositionTranslator) -> (Option<Self>, Vec<Diagnostic>) {
        let parser = grammar::ModuleParser::new();
        let mut errors = Vec::new();
        match parser.parse(&mut errors, &input) {
            Ok(module) => {
                let diagnostics = errors
                    .into_iter()
                    .map(|recovery_error| recovery_error.error)
                    .map(|error| parse_error_to_diagnostic(error, translator))
                    .collect::<Vec<_>>();
                (Some(module), diagnostics)
            }
            Err(fatal_error) => {
                let error = errors
                    .into_iter()
                    .next()
                    .map_or(fatal_error, |recovery_error| recovery_error.error);
                let diagnostics = vec![parse_error_to_diagnostic(error, translator)];
                (None, diagnostics)
            }
        }
    }
}

pub fn parse_error_to_diagnostic(
    error: ParseError<usize, grammar::Token<'_>, &'static str>,
    translator: &PositionTranslator,
) -> Diagnostic {
    use util::Position;
    use ParseError::*;
    let error = error.map_location(|index| translator.position(index));
    let (start, end) = match error {
        InvalidToken { location } | UnrecognizedEOF { location, .. } => (location, location),
        UnrecognizedToken {
            token: (start, _, end),
            ..
        }
        | ExtraToken {
            token: (start, _, end),
        } => (start, end),
        User { .. } => (Position::ORIGIN, Position::ORIGIN),
    };
    Diagnostic {
        range: Range::new(start.to_lsp(), end.to_lsp()),
        severity: Some(DiagnosticSeverity::Error),
        source: Some("rufus".to_string()),
        message: format!("{}", error),
        ..Diagnostic::default()
    }
}
