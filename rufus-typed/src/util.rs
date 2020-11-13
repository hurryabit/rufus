use crate::syntax::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const ORIGIN: Self = Self::new(0, 0);

    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn to_lsp(&self) -> lsp_types::Position {
        lsp_types::Position::new(self.line as u64, self.column as u64)
    }
}
pub struct PositionTranslator {
    line_starts: Vec<usize>,
}

impl PositionTranslator {
    pub fn new(input: &str) -> Self {
        let mut line_starts = Vec::new();
        let mut index = 0;
        line_starts.push(index);
        for line in input.lines() {
            // FIXME(MH): This assumes a newline character is just one byte,
            // which is not true on Windows.
            index += line.len() + 1;
            line_starts.push(index);
        }
        Self { line_starts }
    }

    pub fn position(&self, index: usize) -> Position {
        let line = self
            .line_starts
            .binary_search(&index)
            .unwrap_or_else(|x| x - 1);
        Position::new(line, index - self.line_starts[line])
    }

    pub fn span(&self, span: Span<usize>) -> Span<Position> {
        span.map(|pos| self.position(pos))
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // NOTE(MH): Internally, positions are zero-based. The user gets to see
        // them one-based though.
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_starts() {
        let cases = vec![
            ("", vec![0]),
            ("a", vec![0, 2]),
            ("a\n", vec![0, 2]),
            ("aa", vec![0, 3]),
            ("a\nb", vec![0, 2, 4]),
            ("a\nb\n", vec![0, 2, 4]),
            ("ab\ncd\n", vec![0, 3, 6]),
            ("\na", vec![0, 1, 3]),
        ];
        for (input, expected_line_starts) in cases {
            let trans = PositionTranslator::new(input);
            assert_eq!(trans.line_starts, expected_line_starts);
        }
    }

    #[test]
    fn test_position() {
        let trans = PositionTranslator::new("ab\nc\nde\n\nf");
        let cases = vec![
            (0, 0, 0),
            (1, 0, 1),
            (2, 0, 2),
            (3, 1, 0),
            (4, 1, 1),
            (5, 2, 0),
            (6, 2, 1),
            (7, 2, 2),
            (8, 3, 0),
            (9, 4, 0),
            (10, 4, 1),
            (11, 5, 0),
            (100, 5, 89),
        ];
        for (index, line, col) in cases {
            assert_eq!(trans.position(index), Position::new(line, col));
        }
    }
}
