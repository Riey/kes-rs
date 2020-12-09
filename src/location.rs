use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
}

impl Location {
    pub fn new(line: usize) -> Self {
        Self { line }
    }
    pub fn next_line(mut self) -> Self {
        self.line += 1;
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L{}", self.line)
    }
}
