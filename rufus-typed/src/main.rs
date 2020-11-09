use std::error::Error;

use log::info;
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument, Notification,
        PublishDiagnostics,
    },
    request::GotoDefinition,
    Diagnostic, DiagnosticSeverity, GotoDefinitionResponse, InitializeParams,
    PublishDiagnosticsParams, Range, SaveOptions, ServerCapabilities, TextDocumentItem,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, Url,
};

use lsp_server::{Connection, Message, Request, RequestId, Response};

use rufus_typed::{parser, util};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Set up logging. Because `stdio_transport` gets a lock on stdout and stdin, we must have
    // our logging only write out to stderr.
    flexi_logger::Logger::with_str("info").start().unwrap();
    info!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let text_document_sync = Some(TextDocumentSyncCapability::Options(
        TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::Full),
            save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                include_text: Some(true),
                ..SaveOptions::default()
            })),
            ..TextDocumentSyncOptions::default()
        },
    ));
    let server_capabilities = ServerCapabilities {
        text_document_sync,
        ..ServerCapabilities::default()
    };
    let initialization_params =
        connection.initialize(serde_json::to_value(server_capabilities).unwrap())?;
    main_loop(&connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    info!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: &Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    // info!("_params = {:?}", _params);
    // info!("starting example main loop");
    for msg in &connection.receiver {
        // info!("got msg: {:?}", msg);
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                info!("got request: {:?}", req);
                match cast::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        info!("got gotoDefinition request #{}: {:?}", id, params);
                        let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(_) => (),
                };
                // ...
            }
            Message::Response(resp) => {
                info!("got unhandled response {:?}", resp);
            }
            Message::Notification(not) => {
                match not.method.as_ref() {
                    DidOpenTextDocument::METHOD => {
                        let params = cast_notification::<DidOpenTextDocument>(not);
                        let TextDocumentItem { uri, text, .. } = params.text_document;
                        validate_document(connection, uri, text)?;
                    }
                    DidChangeTextDocument::METHOD => {}
                    DidSaveTextDocument::METHOD => {
                        let params = cast_notification::<DidSaveTextDocument>(not);
                        let uri = params.text_document.uri;
                        match params.text {
                            Some(text) => {
                                validate_document(connection, uri, text)?;
                            }
                            None => {
                                info!("got save notification without text for {}", uri);
                            }
                        }
                    }
                    _ => {
                        info!("got unhandled notification: {:?}", not);
                    }
                };
            }
        }
    }
    Ok(())
}

fn validate_document(
    connection: &Connection,
    uri: Url,
    input: String,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    info!("Received text for {}", &uri);
    let parser = parser::ModuleParser::new();
    let diagnostics = match parser.parse(&input) {
        Ok(_ast) => vec![],
        Err(err) => {
            use lalrpop_util::ParseError::*;
            use util::Position;
            let trans = util::PositionTranslator::new(&input);
            let err = err.map_location(|index| trans.position(index));
            let (start, end) = match err {
                InvalidToken { location } | UnrecognizedEOF { location, .. } => {
                    (location, location)
                }
                UnrecognizedToken {
                    token: (start, _, end),
                    ..
                }
                | ExtraToken {
                    token: (start, _, end),
                } => (start, end),
                User { .. } => (Position::ORIGIN, Position::ORIGIN),
            };
            let diagnostic = Diagnostic {
                range: Range::new(start.to_lsp(), end.to_lsp()),
                severity: Some(DiagnosticSeverity::Error),
                source: Some("rufus".to_string()),
                message: format!("{}", err),
                ..Diagnostic::default()
            };
            vec![diagnostic]
        }
    };

    info!("Sending {} diagnostics", diagnostics.len());
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let not = lsp_server::Notification::new(
        PublishDiagnostics::METHOD.to_string(),
        serde_json::to_value(params).unwrap(),
    );
    connection.sender.send(Message::from(not))?;

    Ok(())
}

fn cast_notification<N>(not: lsp_server::Notification) -> N::Params
where
    N: Notification,
    N::Params: serde::de::DeserializeOwned,
{
    not.extract(N::METHOD).unwrap()
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), Request>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

// #[derive(Debug)]
// enum Error {
//     Io(std::io::Error),
//     Parse(lalrpop_util::ParseError<usize, String, &'static str>),
//     Yaml(serde_yaml::Error),
// }

// impl From<std::io::Error> for Error {
//     fn from(err: std::io::Error) -> Self {
//         Self::Io(err)
//     }
// }

// impl From<lalrpop_util::ParseError<usize, parser::Token<'_>, &'static str>> for Error {
//     fn from(err: lalrpop_util::ParseError<usize, parser::Token<'_>, &'static str>) -> Self {
//         Self::Parse(err.map_token(|t| format!("{}", t)))
//     }
// }

// impl From<serde_yaml::Error> for Error {
//     fn from(err: serde_yaml::Error) -> Self {
//         Self::Yaml(err)
//     }
// }

// fn main() -> Result<(), Error> {
//     let path = if let Some(path) = std::env::args().nth(1) {
//         path
//     } else {
//         panic!("usage: {} <filename>", std::env::args().nth(0).unwrap())
//     };
//     let input = std::fs::read_to_string(path)?;
//     let parser = parser::ModuleParser::new();
//     let ast = parser.parse(&input)?;
//     serde_yaml::to_writer(std::io::stdout(), &ast)?;
//     Ok(())
// }
