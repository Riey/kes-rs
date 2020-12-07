use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
}

impl Location {
    pub fn new(line: usize) -> Self {
        Self { line }
    }
}
