#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Location {
    line: usize,
}

impl Location {
    pub fn new(line: usize) -> Self {
        Self { line }
    }
}
