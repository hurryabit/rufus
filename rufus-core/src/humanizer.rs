use std::fmt;

#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}
#[derive(Debug, Eq, PartialEq)]
pub struct Humanizer {
    line_starts: Vec<usize>,
}

impl Humanizer {
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

    pub fn run(&self, loc: usize) -> SourceLocation {
        let line = self
            .line_starts
            .binary_search(&loc)
            .unwrap_or_else(|x| x - 1);
        SourceLocation {
            line: line as u32,
            column: (loc - self.line_starts[line]) as u32,
        }
    }
}

pub fn sanitize_source_span(msg: &mut String) {
    let regex = regex::Regex::new(r"\d+:\d+(:)\d+:\d+").unwrap();
    if let Some(caps) = regex.captures(msg) {
        let range = caps.get(1).unwrap().range();
        msg.replace_range(range, "-");
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE(MH): Internally, positions are zero-based. The user gets to see
        // them one-based though.
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

impl fmt::Debug for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
            let humanizer = Humanizer::new(input);
            let expected_line_starts: Vec<_> = expected_line_starts.into_iter().collect();
            assert_eq!(humanizer.line_starts, expected_line_starts);
        }
    }

    #[test]
    fn test_translation() {
        let humanizer = Humanizer::new("ab\nc\nde\n\nf");
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
        for (loc, line, column) in cases {
            assert_eq!(humanizer.run(loc), SourceLocation { line, column });
        }
    }
}
