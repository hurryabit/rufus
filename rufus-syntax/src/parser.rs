use logos::Logos;

use crate::kind::{RufusLang, SyntaxExpecation, SyntaxKind, SyntaxKindSet};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ParseError {
    pub span: std::ops::Range<u32>,
    pub found: SyntaxKind,
    pub expected: SyntaxKindSet,
    pub rule: &'static str,
}

/// Stateful parser for the Rufus language.
pub struct Parser<'a> {
    input: &'a str,
    lexer: logos::Lexer<'a, SyntaxKind>,
    peeked: Option<SyntaxKind>,
    follow_stack: Vec<SyntaxKindSet>,
    builder: rowan::GreenNodeBuilder<'a>,
    errors: Vec<ParseError>,
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
            follow_stack: Vec::new(),
            builder: rowan::GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    pub(crate) fn follow(&self) -> SyntaxKindSet {
        self.follow_stack.last().copied().unwrap_or_default()
    }

    pub(crate) fn checkpoint(&mut self) -> rowan::Checkpoint {
        self.peek(); // Put whitespace before the checkpoint.
        self.builder.checkpoint()
    }

    pub(crate) fn error(&mut self, found: SyntaxKind, expected: impl SyntaxExpecation, rule: &'static str) {
        let span = self.lexer.span();
        self.errors.push(ParseError {
            span: span.start as u32..span.end as u32,
            found,
            expected: expected.to_set(),
            rule,
        });
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

    pub(crate) fn consume(&mut self, expected: impl SyntaxExpecation) {
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

    pub(crate) fn find(&mut self, expected: impl SyntaxExpecation, rule: &'static str) -> bool {
        let parser = self;
        let mut token = parser.peek();
        if expected.contains(token) {
            return true;
        }
        parser.error(token, expected, rule);
        let mut parser = parser.with_node(SyntaxKind::ERROR);
        let follow = parser.follow();
        while !expected.contains(token) && !follow.contains(token) {
            parser.consume(token);
            token = parser.peek();
        }
        expected.contains(token)
    }

    pub(crate) fn find_and_consume(&mut self, expected: impl SyntaxExpecation, rule: &'static str) {
        if self.find(expected, rule) {
            self.consume(expected);
        }
    }

    pub(crate) fn with_node<'b>(&'b mut self, kind: SyntaxKind) -> NodeScope<'a, 'b> {
        NodeScope::new(self, kind)
    }

    pub(crate) fn with_node_at<'b>(&'b mut self, checkpoint: rowan::Checkpoint, kind: SyntaxKind) -> NodeScope<'a, 'b> {
        NodeScope::new_at_checkpoint(self, checkpoint, kind)
    }

    pub(crate) fn with_follow<'b>(&'b mut self, kinds: impl SyntaxExpecation) -> FollowScope<'a, 'b> {
        FollowScope::new(self, kinds)
    }
}

pub struct NodeScope<'a, 'b> {
    parser: &'b mut Parser<'a>,
}

impl<'a, 'b> NodeScope<'a, 'b> {
    fn new(parser: &'b mut Parser<'a>, kind: SyntaxKind) -> Self {
        parser.builder.start_node(kind.into());
        Self { parser }
    }

    fn new_at_checkpoint(parser: &'b mut Parser<'a>, checkpoint: rowan::Checkpoint, kind: SyntaxKind) -> Self {
        parser.builder.start_node_at(checkpoint, kind.into());
        Self { parser }
    }
}

impl<'a, 'b> std::ops::Deref for NodeScope<'a, 'b> {
    type Target = Parser<'a>;

    fn deref(&self) -> &Self::Target {
        self.parser
    }
}

impl<'a, 'b> std::ops::DerefMut for NodeScope<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.parser
    }
}

impl<'a, 'b> Drop for NodeScope<'a, 'b> {
    fn drop(&mut self) {
        self.parser.builder.finish_node();
    }
}

pub struct FollowScope<'a, 'b> {
    parser: &'b mut Parser<'a>,
}

impl<'a, 'b> FollowScope<'a, 'b> {
    fn new(parser: &'b mut Parser<'a>, kinds: impl SyntaxExpecation) -> Self {
        let top = parser.follow();
        parser
            .follow_stack
            .push(SyntaxKindSet::union([top, kinds.to_set()]));
        Self { parser }
    }
}

impl<'a, 'b> std::ops::Deref for FollowScope<'a, 'b> {
    type Target = Parser<'a>;

    fn deref(&self) -> &Self::Target {
        self.parser
    }
}

impl<'a, 'b> std::ops::DerefMut for FollowScope<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.parser
    }
}

impl<'a, 'b> Drop for FollowScope<'a, 'b> {
    fn drop(&mut self) {
        let top = self.parser.follow_stack.pop();
        assert!(top.is_some());
    }
}
