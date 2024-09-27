#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
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

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        self == Self::WHITESPACE || self == Self::COMMENT
    }

    pub fn as_set(self) -> SyntaxKindSet {
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
        assert!(raw.0 <= SyntaxKind::ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    pub fn to_vec(self) -> Vec<SyntaxKind> {
        let mut kinds = Vec::new();
        let mut i = 0;
        while i <= SyntaxKind::ROOT as u16 {
            if self.0 & (1 << i) != 0 {
                kinds.push(unsafe { std::mem::transmute::<u16, SyntaxKind>(i) });
            }
            i += 1;
        }
        kinds
    }
}

impl std::fmt::Debug for SyntaxKindSet {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_set().entries(self.to_vec()).finish()
    }
}

pub(crate) trait SyntaxExpecation: Copy + std::fmt::Debug {
    fn contains(&self, kind: SyntaxKind) -> bool;
    fn to_set(&self) -> SyntaxKindSet;
}

impl SyntaxExpecation for SyntaxKind {
    fn contains(&self, kind: SyntaxKind) -> bool {
        *self == kind
    }
    fn to_set(&self) -> SyntaxKindSet {
        SyntaxKindSet::singleton(*self)
    }
}

impl SyntaxExpecation for SyntaxKindSet {
    fn contains(&self, kind: SyntaxKind) -> bool {
        self.0 & (1 << (kind as u64)) != 0
    }

    fn to_set(&self) -> SyntaxKindSet {
        *self
    }
}

pub type SyntaxNode = rowan::SyntaxNode<RufusLang>;
pub type SyntaxElement = rowan::SyntaxElement<RufusLang>;

pub fn dump_syntax(root: SyntaxNode, include_ws: bool) -> String {
    fn go(node: SyntaxNode, buffer: &mut String, indent: &mut String, include_ws: bool) {
        buffer.push_str(&format!("{}{:?}\n", indent, node));
        indent.push_str("  ");
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(node) => go(node, buffer, indent, include_ws),
                SyntaxElement::Token(token) => {
                    if include_ws || !token.kind().is_trivia() {
                        buffer.push_str(&format!("{}#{:?}\n", indent, token));
                    }
                }
            }
        }
        indent.truncate(indent.len() - 2);
    }

    let mut buffer = String::new();
    go(root, &mut buffer, &mut String::new(), include_ws);
    // panic!("BUFFER: {}", buffer);
    buffer
}
