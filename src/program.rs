use crate::compiler::Compiler;
use crate::error::ParseError;
use crate::instruction::InstructionWithDebug;
use crate::interner::{Interner, Symbol};
use crate::parser::parse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Program {
    interner: Interner,
    instructions: Vec<InstructionWithDebug>,
}

impl Program {
    pub fn from_source(source: &str) -> Result<Self, ParseError> {
        let mut interner = Interner::new();
        let ast = parse(source, &mut interner)?;

        Ok(Self {
            interner,
            instructions: Compiler::new().compile(&ast),
        })
    }

    #[inline]
    pub fn instructions(&self) -> &[InstructionWithDebug] {
        &self.instructions
    }

    #[inline]
    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.interner.resolve(symbol)
    }
}
