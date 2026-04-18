#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub const fn single(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    pub const fn merge(self, other: Span) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }
}
