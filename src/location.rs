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
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L{}", self.line)
    }
}
