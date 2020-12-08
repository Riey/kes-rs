use crate::ast::Stmt;
use crate::compiler::Compiler;
use crate::error::ParseError;
use crate::instruction::InstructionWithDebug;
use crate::interner::{Interner, Symbol};
use crate::parser::parse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Program {
    interner: Interner,
    instructions: Vec<InstructionWithDebug>,
}

impl Program {
    pub fn from_ast(program: &[Stmt], interner: Interner) -> Self {
        Self {
            interner,
            instructions: Compiler::new().compile(program),
        }
    }

    pub fn from_source(source: &str) -> Result<Self, ParseError> {
        let mut interner = Interner::new();
        let ast = parse(source, &mut interner)?;

        Ok(Self::from_ast(&ast, interner))
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

#[cfg(test)]
mod tests {
    use super::Program;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_serde() {
        let prev = Program::from_source("만약 1 { ㅇ(1+2*3, 4); } 그외 { 123; }").unwrap();
        let bytes = bincode::serialize(&prev).unwrap();
        let cur = bincode::deserialize::<Program>(&bytes).unwrap();
        assert_eq!(prev, cur);
    }
}
