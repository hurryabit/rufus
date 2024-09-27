use logos::Logos;

use crate::kind::{RufusLang, SyntaxKind, SyntaxKindSet};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ParseError {
    pub span: std::ops::Range<u32>,
    pub found: SyntaxKind,
    pub expected: SyntaxKindSet,
    pub rule: &'static str,
}

/// Stateful parser for the Rufus language.
pub struct Parser<'a> {
    pub input: &'a str,
    pub lexer: logos::Lexer<'a, SyntaxKind>,
    pub peeked: Option<SyntaxKind>,
    pub builder: rowan::GreenNodeBuilder<'a>,
    pub errors: Vec<ParseError>,
}

pub struct ParseResult {
    pub syntax: rowan::SyntaxNode<RufusLang>,
    pub errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    /// Create a new parser on the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            lexer: SyntaxKind::lexer(input),
            peeked: None,
            builder: rowan::GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self, rule: fn(&mut Parser)) -> ParseResult {
        rule(&mut self);
        let green_node = self.builder.finish();
        ParseResult {
            syntax: rowan::SyntaxNode::new_root(green_node),
            errors: self.errors,
        }
    }

    /// Peek the `SyntaxKind` of the next non-trivia token.
    pub(crate) fn peek(&mut self) -> SyntaxKind {
        if let Some(token) = self.peeked {
            return token;
        }
        let token = loop {
            match self.lexer.next() {
                None => break SyntaxKind::EOF,
                Some(Err(_)) => break SyntaxKind::UNKNOWN,
                Some(Ok(token)) => {
                    if token.is_trivia() {
                        self.builder
                            .token(token.into(), &self.input[self.lexer.span()]);
                    } else {
                        break token;
                    }
                }
            }
        };
        self.peeked = Some(token);
        token
    }

    pub(crate) fn assert_first(&mut self, first: SyntaxKindSet) {
        assert!(first.contains(self.peek()));
    }

    /// Consume the next token.
    pub(crate) fn consume(&mut self, expected: SyntaxKind) {
        self.consume_in(expected.as_set());
    }

    pub(crate) fn consume_in(&mut self, expected: SyntaxKindSet) {
        match self.peeked {
            None => panic!("consume without peek"),
            Some(token) => {
                if !expected.contains(token) {
                    panic!("consumed {:?}, but expected {:?}", token, expected)
                }
                if token == SyntaxKind::EOF {
                    panic!("consume end-of-file");
                }
                self.builder
                    .token(token.into(), &self.input[self.lexer.span()]);
                self.peeked = None;
            }
        }
    }

    pub(crate) fn error(&mut self, found: SyntaxKind, expected: SyntaxKindSet, rule: &'static str) {
        let span = self.lexer.span();
        self.errors.push(ParseError {
            span: span.start as u32..span.end as u32,
            found,
            expected,
            rule,
        });
    }

    // `first` is the FIRST set of the next symbol in the rule we're parsing.
    // `follow` is the FOLLOW set of the non-terminal whose rule we're parsing.
    // Returns whether we can continue parsing the rule.
    pub(crate) fn expect(
        &mut self,
        expected: SyntaxKind,
        first: SyntaxKindSet,
        follow: SyntaxKindSet,
        rule: &'static str,
    ) -> bool {
        let mut token = self.peek();
        if token == expected {
            self.consume(expected);
            return true;
        }
        self.error(token, expected.as_set(), rule);
        self.builder.start_node(SyntaxKind::ERROR.into());
        while token != expected && !first.contains(token) && !follow.contains(token) {
            self.consume(token);
            token = self.peek();
        }
        self.builder.finish_node();
        if token == expected {
            self.consume(expected);
            return true;
        }
        first.contains(token)
    }
}
