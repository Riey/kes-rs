use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use string_interner::StringInterner;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Symbol(NonZeroU32);

impl string_interner::symbol::Symbol for Symbol {
    fn try_from_usize(index: usize) -> Option<Self> {
        NonZeroU32::new((index as u32).wrapping_add(1)).map(Self)
    }

    fn to_usize(self) -> usize {
        self.0.get() as usize - 1
    }
}

pub type Interner = StringInterner<Symbol>;
