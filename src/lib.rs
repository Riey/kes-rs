mod instruction;
mod lexer;
mod operator;
mod parser;
mod token;

use std::collections::HashMap;
use std::convert::TryFrom;

pub use crate::instruction::Instruction;
pub use crate::operator::Operator;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Value {
    Int(u32),
    Str(String),
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        if b {
            Value::Int(1)
        } else {
            Value::Int(0)
        }
    }
}

impl From<Value> for bool {
    fn from(v: Value) -> Self {
        match v {
            Value::Int(0) => false,
            Value::Str(s) if s.is_empty() => false,
            _ => true,
        }
    }
}

impl From<u32> for Value {
    fn from(n: u32) -> Self {
        Value::Int(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl<'s> From<&'s str> for Value {
    fn from(s: &'s str) -> Self {
        Value::Str(s.into())
    }
}

#[derive(Debug)]
pub struct ValueConvertError;

impl TryFrom<Value> for u32 {
    type Error = ValueConvertError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n),
            _ => Err(ValueConvertError),
        }
    }
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

/*
pub struct Context<'c> {
    builtin: &'c HashMap<String, fn(&mut Self)>,
    stack: &'c mut Vec<Value>,
}

impl<'c> Context<'c> {
    pub fn push(&mut self, v: impl Into<Value>) {
        self.stack.push(v.into());
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn run_operator(&mut self, op: Operator) {
        macro_rules! binop {
            ($op:tt) => {
                self.pop().unwrap() $op self.pop().unwrap()
            };
        }

        macro_rules! bin_intop {
            ($op:tt) => {
                u32::try_from(self.pop().unwrap()).unwrap() $op u32::try_from(self.pop().unwrap()).unwrap()
            };
        }

        match op {
            Operator::Boolean(op) => {
                let b = match op {
                    BooleanOperator::Equal => binop!(==),
                    BooleanOperator::NotEqual => binop!(!=),
                    BooleanOperator::Greater => binop!(>),
                    BooleanOperator::Less => binop!(<),
                    BooleanOperator::GreaterOrEqual => binop!(>=),
                    BooleanOperator::LessOrEqual => binop!(<=),
                    BooleanOperator::Not => {
                        let b: bool = self.pop().unwrap().into();
                        !b
                    }
                };

                self.push(b);
            }

            Operator::Simple(op) => {
                let n = match op {
                    SimpleOperator::Add => bin_intop!(+),
                    SimpleOperator::Sub => bin_intop!(-),
                    SimpleOperator::Mul => bin_intop!(*),
                    SimpleOperator::Div => bin_intop!(/),
                    SimpleOperator::Rem => bin_intop!(%),
                    SimpleOperator::And => bin_intop!(&),
                    SimpleOperator::Or => bin_intop!(|),
                    SimpleOperator::Xor => bin_intop!(^),
                };

                self.push(n);
            }
        }
    }

    pub fn run_instruction<'s>(&mut self, inst: Instruction<'s>) {
        match inst {
            Instruction::Operator(op) => {}
        }
    }
}

pub struct Interpreter {
    builtin: HashMap<String, fn(&mut Context)>,
}

impl Interpreter {
    pub fn new(builtin: HashMap<String, fn(&mut Context)>) -> Self {
        Self { builtin }
    }

    pub fn run<'s>(&self, script: &Script<'s>) {
        let mut stack = Vec::with_capacity(100);

        let mut ctx = Context {
            builtin: &self.builtin,
            stack: &mut stack,
        };

        for &inst in script.instructions() {
            ctx.run_instruction(inst);
        }
    }
}

//    stack: Vec<Value>,
//}

*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
