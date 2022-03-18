#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Default)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn newline(&mut self) {
        self.line += 1;
        self.col = 1;
    }

    pub fn advance_col(&mut self, by: usize) {
        self.col += by;
    }
}
