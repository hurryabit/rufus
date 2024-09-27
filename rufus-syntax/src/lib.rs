// This module contains a parser for the following context-free grammar with
// start symbol ROOT (rules are indented for the sake of visual grouing only):
//
// ROOT -> EXPR
// EXPR -> FUN_EXPR | LET_EXPR | IF_EXPR | SUM_EXPR
// FUN_EXPR -> "fun" PARAM_LIST "->" EXPR
//   PARAM_LIST -> PARAM*
//   PARAM -> ID_LOWER
// LET_EXPR -> "let" LET_MOD LET_VAR "=" EXPR "in" EXPR
//   LET_MOD -> "rec"?
//   LET_VAR -> ID_LOWER
// IF_EXPR -> "if" EXPR "then" EXPR "else" EXPR
// SUM_EXPR -> PROD_EXPR | SUM_EXPR SUM_OP PROD_EXPR
//   SUM_OP -> "+" | "-"
// PROD_EXPR -> ATOM_EXPR | PROD_EXPR PROD_OP ATOM_EXPR
//   PROD_OP -> "*" | "/"
// ATOM_EXPR -> VAR_EXPR | LIT_EXPR | PAREN_EXPR
// VAR_EXPR -> ID_LOWER
// LIT_EXPR -> NUM_LIT | TRUE | FALSE
// PAREN_EXPR -> "(" EXPR ")"
//
// The resulting CST does not contain nodes for EXPR and ATOM_EXPR but rather
// just their immediate children. SUM_EXPR and PROD_EXPR use a common node
// type BINOP_EXPR. Similarly, SUM_OP and PROD_OP are fused into BINOP.


use std::fmt::Debug;

use logos::Logos;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, logos::Logos)]
#[repr(u16)]
pub enum SyntaxKind {
    // Terminals aka tokens
    #[token("fun")]
    FUN = 0,
    #[token("let")]
    LET,
    #[token("rec")]
    REC,
    #[token("in")]
    IN,
    #[token("if")]
    IF,
    #[token("then")]
    THEN,
    #[token("else")]
    ELSE,
    #[token("true")]
    TRUE,
    #[token("false")]
    FALSE,

    #[token("->")]
    ARROW,
    #[token("=")]
    ASSIGN,
    #[token("(")]
    LPAREN,
    #[token(")")]
    RPAREN,
    #[token(".")]
    DOT,
    #[token(",")]
    COMMA,
    #[token("+")]
    PLUS,
    #[token("-")]
    MINUS,
    #[token("*")]
    STAR,
    #[token("/")]
    SLASH,
    #[token("==")]
    EQ,
    #[token("!=")]
    NE,
    #[token("<")]
    LT,
    #[token("<=")]
    LE,
    #[token(">")]
    GT,
    #[token(">=")]
    GE,

    #[regex(r"[a-z][a-zA-Z0-9_]*")]
    ID_LOWER,
    #[regex(r"[0-9]+")]
    NAT_LIT,
    #[regex(r"\s+")]
    WHITESPACE,
    #[regex(r"\(\*[^\*]*\*\)")] // TODO(MH): This is too restrictive.
    COMMENT,

    // Special tokens
    UNKNOWN, // Unknown token, used for error recovery.
    EOF,     // End-of-file.

    BINOP,

    PARAM,
    PARAM_LIST,

    BINOP_EXPR,
    APP_EXPR,
    VAR_EXPR,
    LIT_EXPR,
    PAREN_EXPR,

    LET_MOD,
    LET_VAR,

    LET_EXPR,
    FUN_EXPR,

    ERROR,
    ROOT,
}

use SyntaxKind::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKindSet(u64);

impl SyntaxKindSet {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn singleton(kind: SyntaxKind) -> Self {
        Self(1 << (kind as u64))
    }

    pub const fn from<const N: usize>(kinds: [SyntaxKind; N]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < N {
            bits |= 1 << (kinds[i] as u64);
            i += 1;
        }
        Self(bits)
    }

    pub const fn union<const N: usize>(sets: [SyntaxKindSet; N]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < N {
            bits |= sets[i].0;
            i += 1;
        }
        Self(bits)
    }

    pub fn contains(self, kind: SyntaxKind) -> bool {
        self.0 & (1 << (kind as u64)) != 0
    }

    pub fn to_vec(self) -> Vec<SyntaxKind> {
        let mut kinds = Vec::new();
        let mut i = 0;
        while i <= ROOT as u16 {
            if self.0 & (1 << i) != 0 {
                kinds.push(unsafe { std::mem::transmute::<u16, SyntaxKind>(i) });
            }
            i += 1;
        }
        kinds
    }
}

// Token classes.
const TRIVIA: SyntaxKindSet = SyntaxKindSet::from([WHITESPACE, COMMENT]);
const ADD_OPS: SyntaxKindSet = SyntaxKindSet::from([PLUS, MINUS]);
const MUL_OPS: SyntaxKindSet = SyntaxKindSet::from([STAR, SLASH]);
const CMP_OPS: SyntaxKindSet = SyntaxKindSet::from([EQ, NE, LT, LE, GT, GE]);
const BIN_OPS: SyntaxKindSet = SyntaxKindSet::union([ADD_OPS, MUL_OPS, CMP_OPS]);
const LITERAL: SyntaxKindSet = SyntaxKindSet::from([NAT_LIT, TRUE, FALSE]);

// First sets.
const FIRST_ATOM_EXPR: SyntaxKindSet =
    SyntaxKindSet::union([SyntaxKindSet::from([ID_LOWER, LPAREN]), LITERAL]);
const FIRST_EXPR: SyntaxKindSet =
    SyntaxKindSet::union([SyntaxKindSet::from([FUN, LET, IF]), FIRST_ATOM_EXPR]);

// Follow sets.
const FOLLOW_EXPR: SyntaxKindSet = SyntaxKindSet::union([
    SyntaxKindSet::from([RPAREN, DOT, IN, THEN, ELSE, EOF]),
    BIN_OPS,
    FIRST_ATOM_EXPR, // Because function application is juxtaposition.
]);
const FOLLOW_PARAM_LIST: SyntaxKindSet = SyntaxKindSet::from([ARROW]);
const FOLLOW_PARAM: SyntaxKindSet =
    SyntaxKindSet::union([FOLLOW_PARAM_LIST, SyntaxKindSet::from([ID_LOWER])]);

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        return TRIVIA.contains(self);
    }

    fn as_set(self) -> SyntaxKindSet {
        SyntaxKindSet::singleton(self)
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RufusLang {}

impl rowan::Language for RufusLang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

#[derive(Debug)]
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
            builder: rowan::GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self) -> ParseResult {
        self.builder.start_node(ROOT.into());
        self.expr();
        let mut token = self.peek();
        if token != EOF {
            self.error(token, EOF.as_set(), "root");
            self.builder.start_node(ERROR.into());
            while token != EOF {
                self.consume(token);
                token = self.peek();
            }
            self.builder.finish_node();
        }
        self.builder.finish_node();
        let green_node = self.builder.finish();
        ParseResult {
            syntax: rowan::SyntaxNode::new_root(green_node),
            errors: self.errors,
        }
    }

    /// Peek the `SyntaxKind` of the next non-trivia token.
    fn peek(&mut self) -> SyntaxKind {
        if let Some(token) = self.peeked {
            return token;
        }
        let token = loop {
            match self.lexer.next() {
                None => break EOF,
                Some(Err(_)) => break UNKNOWN,
                Some(Ok(token)) => {
                    if TRIVIA.contains(token) {
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

    /// Consume the next token.
    fn consume(&mut self, expected: SyntaxKind) {
        self.consume_in(expected.as_set());
    }

    fn consume_in(&mut self, expected: SyntaxKindSet) {
        match self.peeked {
            None => panic!("consume without peek"),
            Some(token) => {
                if !expected.contains(token) {
                    panic!("consumed {:?}, but expected {:?}", token, expected)
                }
                self.builder
                    .token(token.into(), &self.input[self.lexer.span()]);
                self.peeked = None;
            }
        }
    }

    fn error(&mut self, found: SyntaxKind, expected: SyntaxKindSet, rule: &'static str) {
        let span = self.lexer.span();
        self.errors.push(ParseError {
            span: span.start as u32..span.end as u32,
            found,
            expected,
            rule,
        });
    }

    // EXPR -> FUN_EXPR | LET_EXPR | IF_EXPR | SUM_EXPR
    fn expr(&mut self) {
        match self.peek() {
            FUN => self.fun_expr(),
            LET => self.let_expr(),
            IF => self.if_expr(),
            _ => self.sum_expr(),
        }
    }

    // FUN_EXPR -> "fun" PARAM_LIST "->" EXPR
    fn fun_expr(&mut self) {
        self.builder.start_node(FUN_EXPR.into());
        self.consume(FUN);
        self.param_list();
        let mut token = self.peek();
        if token != ARROW {
            self.error(token, ARROW.as_set(), "fun_expr");
            self.builder.start_node(ERROR.into());
            while !FOLLOW_EXPR.contains(token) && token != ARROW {
                self.consume(token);
                token = self.peek();
            }
            self.builder.finish_node();
            if token != ARROW {
                self.builder.finish_node();
                return;
            }
        }
        self.consume(ARROW);
        self.expr();
        self.builder.finish_node();
    }

    // PARAM_LIST -> PARAM*
    // PARAM -> ID_LOWER
    fn param_list(&mut self) {
        self.builder.start_node(PARAM_LIST.into());
        loop {
            match self.peek() {
                ID_LOWER => {
                    self.builder.start_node(PARAM.into());
                    self.consume(ID_LOWER);
                    self.builder.finish_node();
                }
                ARROW => break,
                mut token => {
                    assert!(!FOLLOW_PARAM.contains(token));
                    self.error(token, FOLLOW_PARAM, "param_list");
                    self.builder.start_node(ERROR.into());
                    while !FOLLOW_PARAM.contains(token) {
                        self.consume(token);
                        token = self.peek();
                    }
                    self.builder.finish_node();
                }
            }
        }
        self.builder.finish_node();
    }

    // LET_EXPR -> "let" LET_MOD LET_VAR "=" EXPR "in" EXPR
    // LET_MOD -> "rec"?
    fn let_expr(&mut self) {
        self.builder.start_node(LET_EXPR.into());
        self.consume(LET);

        self.builder.start_node(LET_MOD.into());
        if self.peek() == REC {
            self.consume(REC);
        }
        self.builder.finish_node();
        self.let_var();

        let mut token = self.peek();
        if token != ASSIGN {
            self.error(token, ASSIGN.as_set(), "LET_EXPR");
            self.builder.start_node(ERROR.into());
            while token != ASSIGN && !FIRST_EXPR.contains(token) && !FOLLOW_EXPR.contains(token) {
                self.consume(token);
                token = self.peek();
            }
            self.builder.finish_node();
            if token == ASSIGN {
                self.consume(ASSIGN);
            } else if !FIRST_EXPR.contains(token) {
                self.builder.finish_node();
                return
            }
        } else {
            self.consume(ASSIGN);
        }

        self.expr();

        let mut token = self.peek();
        if token != IN {
            self.error(token, ASSIGN.as_set(), "LET_EXPR");
            self.builder.start_node(ERROR.into());
            while token != ASSIGN && !FIRST_EXPR.contains(token) && !FOLLOW_EXPR.contains(token) {
                self.consume(token);
                token = self.peek();
            }
            self.builder.finish_node();
            if token == ASSIGN {
                self.consume(IN);
            } else if !FIRST_EXPR.contains(token) {
                self.builder.finish_node();
                return
            }
        } else {
            self.consume(IN);
        }

        self.expr();

        self.builder.finish_node();
    }

    // LET_VAR -> ID_LOWER
    fn let_var(&mut self) {
        self.builder.start_node(LET_VAR.into());
        let mut token = self.peek();
        if token != ID_LOWER {
            self.error(token, ID_LOWER.as_set(), "LET_VAR");
            while token != ID_LOWER && token != ASSIGN {
                self.consume(token);
                token = self.peek();
            }
            self.builder.finish_node();
            if token != ID_LOWER {
                self.builder.finish_node();
                return
            }
        }
        self.consume(ID_LOWER);
        self.builder.finish_node();
    }

    // IF_EXPR -> "if" EXPR "then" EXPR "else" EXPR
    fn if_expr(&mut self) {
        todo!()
    }

    // SUM_EXPR -> PROD_EXPR | SUM_EXPR SUM_OP PROD_EXPR
    // SUM_OP -> "+" | "-"
    fn sum_expr(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.prod_expr(); // TODO(MH): This needs to be product_expr here and down.
        let mut token = self.peek();
        while ADD_OPS.contains(token) {
            self.builder.start_node_at(checkpoint, BINOP_EXPR.into());
            self.builder.start_node(BINOP.into());
            self.consume(token);
            self.builder.finish_node();
            self.prod_expr();
            self.builder.finish_node();
            token = self.peek();
        }
    }

    // PROD_EXPR -> ATOM_EXPR | PROD_EXPR PROD_OP ATOM_EXPR
    // PROD_OP -> "*" | "/"
    fn prod_expr(&mut self) {
        let checkpoint = self.builder.checkpoint();
        self.atom_expr(); // TODO(MH): This needs to be product_expr here and down.
        let mut token = self.peek();
        while MUL_OPS.contains(token) {
            self.builder.start_node_at(checkpoint, BINOP_EXPR.into());
            self.builder.start_node(BINOP.into());
            self.consume(token);
            self.builder.finish_node();
            self.atom_expr();
            self.builder.finish_node();
            token = self.peek();
        }
    }

    // fn app_expr(&mut self) {
    //     todo!()
    // }

    // ATOM_EXPR -> VAR_EXPR | LIT_EXPR | PAREN_EXPR
    fn atom_expr(&mut self) {
        let mut token = self.peek();
        if !FIRST_ATOM_EXPR.contains(token) {
            self.error(token, FIRST_ATOM_EXPR, "atom_expr");
            self.builder.start_node(ERROR.into());
            while !FIRST_ATOM_EXPR.contains(token) && !FOLLOW_EXPR.contains(token) {
                self.consume(token);
                token = self.peek()
            }
            self.builder.finish_node();
            if !FIRST_ATOM_EXPR.contains(token) {
                return;
            }
        }
        match self.peek() {
            ID_LOWER => self.var_expr(),
            NAT_LIT | TRUE | FALSE => self.lit_expr(),
            LPAREN => self.paren_expr(),
            _ => unreachable!(),
        }
    }

    // VAR_EXPR -> ID_LOWER
    fn var_expr(&mut self) {
        self.builder.start_node(VAR_EXPR.into());
        self.consume(ID_LOWER);
        self.builder.finish_node();
    }

    // LIT_EXPR -> LITERAL
    // LITERAL -> NUM_LIT | TRUE | FALSE
    fn lit_expr(&mut self) {
        self.builder.start_node(LIT_EXPR.into());
        self.consume_in(LITERAL);
        self.builder.finish_node();
    }

    // PAREN_EXPR -> "(" EXPR ")"
    fn paren_expr(&mut self) {
        self.builder.start_node(PAREN_EXPR.into());
        self.consume(LPAREN);
        self.expr();
        let mut token = self.peek();
        if token != RPAREN {
            self.error(token, RPAREN.as_set(), "paren_expr");
            self.builder.start_node(ERROR.into());
            while token != RPAREN && !FOLLOW_EXPR.contains(token) {
                self.consume(token);
                token = self.peek();
            }
            self.builder.finish_node();
            if token != RPAREN {
                self.builder.finish_node();
                return;
            }
        }
        self.consume(RPAREN);
        self.builder.finish_node();
    }
}

impl std::fmt::Debug for SyntaxKindSet {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_set().entries(self.to_vec()).finish()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<RufusLang>;
pub type SyntaxElement = rowan::SyntaxElement<RufusLang>;
