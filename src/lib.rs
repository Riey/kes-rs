mod instruction;
mod lexer;
mod operator;
mod parser;
mod token;

pub use crate::instruction::Instruction;
pub use crate::operator::{BooleanOperator, Operator, SimpleOperator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Int(u32),
    Str(String),
}

pub struct Script<'s> {
    instructions: Vec<Instruction<'s>>,
}

impl<'s> Script<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            instructions: crate::parser::parse(crate::lexer::lex(source)),
        }
    }

    pub fn instructions(&self) -> &[Instruction<'s>] {
        &self.instructions
    }
}

//pub struct Context {
//    vars: BTreeMap<String, Value>,
//    builtin_functions: BTreeMap<String, fn(&mut Self)>,
//    stack: Vec<Value>,
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
