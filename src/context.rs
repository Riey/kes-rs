use crate::instruction::Instruction;
use crate::operator::Operator;
use crate::printer::Printer;
use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Value<'b> {
    Int(u32),
    Str(&'b str),
}

impl<'b> Value<'b> {
    #[inline]
    pub fn into_bool(self) -> bool {
        self.into()
    }
}

impl<'b> fmt::Display for Value<'b> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(num) => num.fmt(formatter),
            Value::Str(str) => formatter.write_str(str),
        }
    }
}

impl<'b> From<bool> for Value<'b> {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Value::Int(1)
        } else {
            Value::Int(0)
        }
    }
}

impl<'b> From<Value<'b>> for bool {
    #[inline]
    fn from(v: Value) -> Self {
        match v {
            Value::Int(0) | Value::Str("") => false,
            _ => true,
        }
    }
}

impl<'b> From<u32> for Value<'b> {
    #[inline]
    fn from(n: u32) -> Self {
        Value::Int(n)
    }
}

impl<'b> From<&'b str> for Value<'b> {
    #[inline]
    fn from(s: &'b str) -> Self {
        Value::Str(s)
    }
}

#[derive(Debug)]
pub struct ValueConvertError;

impl<'b> TryFrom<Value<'b>> for u32 {
    type Error = ValueConvertError;

    fn try_from(v: Value<'b>) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n),
            _ => Err(ValueConvertError),
        }
    }
}

pub struct Context<'b: 'c, 'c, P: Printer> {
    bump: &'c Bump,
    builtin: &'c HashMap<&'b str, fn(&mut Context<'b, 'c, P>)>,
    instructions: &'c [Instruction<'b>],
    printer: &'c mut P,
    print_buffer: &'c mut String<'b>,
    stack: Vec<'c, Value<'c>>,
    variables: HashMap<&'c str, Value<'c>>,
    cursor: usize,
}

impl<'b: 'c, 'c, P: Printer> Context<'b, 'c, P> {
    pub fn new(
        bump: &'c Bump,
        builtin: &'c HashMap<&'b str, fn(&mut Context<'b, 'c, P>)>,
        instructions: &'c [Instruction<'b>],
        printer: &'c mut P,
        print_buffer: &'c mut String<'b>,
    ) -> Self {
        Self {
            bump,
            builtin,
            instructions,
            stack: Vec::with_capacity_in(50, bump),
            printer,
            print_buffer,
            variables: HashMap::new(),
            cursor: 0,
        }
    }

    pub fn push(&mut self, v: impl Into<Value<'c>>) {
        self.stack.push(v.into());
    }

    pub fn pop(&mut self) -> Option<Value<'c>> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&Value<'c>> {
        self.stack.last()
    }

    fn pop_u32(&mut self) -> u32 {
        self.pop().unwrap().try_into().unwrap()
    }

    fn pop_bool(&mut self) -> bool {
        self.pop().unwrap().into()
    }

    pub fn run_operator(&mut self, op: Operator) {
        macro_rules! binop {
            ($op:tt) => {
                let rhs = self.pop_u32();
                let lhs = self.pop_u32();
                self.push(lhs $op rhs);
            };
        }

        macro_rules! binop_bool {
            ($op:tt) => {
                let rhs = self.pop_u32();
                let lhs = self.pop_u32();
                self.push(if lhs $op rhs {
                    1
                } else {
                    0
                });
            };
        }

        macro_rules! binop_raw_bool {
            ($op:tt) => {
                let rhs = self.pop().unwrap();
                let lhs = self.pop().unwrap();
                self.push(if lhs $op rhs {
                    1
                } else {
                    0
                });
            }
        }

        match op {
            Operator::Equal => {
                binop_raw_bool!(==);
            }
            Operator::NotEqual => {
                binop_raw_bool!(!=);
            }
            Operator::Greater => {
                binop_bool!(>);
            }
            Operator::Less => {
                binop_bool!(<);
            }
            Operator::GreaterOrEqual => {
                binop_bool!(>=);
            }
            Operator::LessOrEqual => {
                binop_bool!(<=);
            }
            Operator::Not => {
                let b = self.pop_bool();

                self.push(if b { 1 } else { 0 });
            }
            Operator::Add => {
                binop!(+);
            }
            Operator::Sub => {
                binop!(-);
            }
            Operator::Mul => {
                binop!(*);
            }
            Operator::Div => {
                binop!(/);
            }
            Operator::Rem => {
                binop!(%);
            }
            Operator::And => {
                binop!(&);
            }
            Operator::Or => {
                binop!(|);
            }
            Operator::Xor => {
                binop!(^);
            }
        }
    }

    pub fn flush_print(&mut self) {
        self.print_buffer.clear();
        for v in self.stack.drain(..) {
            write!(&mut self.print_buffer, "{}", v).unwrap();
        }
        self.printer.print(&self.print_buffer);
    }

    #[inline(always)]
    pub fn bump(&self) -> &'c Bump {
        self.bump
    }

    pub fn run_instruction(&mut self, inst: Instruction<'b>) -> bool {
        match inst {
            Instruction::LoadInt(num) => self.push(num),
            Instruction::LoadStr(str) => self.push(str),
            Instruction::LoadVar(name) => {
                let item = self.variables[name];
                self.push(item);
            }
            Instruction::StoreVar(name) => {
                let item = self.pop().unwrap();
                self.variables.insert(name, item);
            }
            Instruction::CallBuiltin(name) => self.run_builtin(name),
            Instruction::Operator(op) => self.run_operator(op),
            Instruction::Goto(pos) => {
                self.cursor = pos;
                return false;
            }
            Instruction::GotoIfNot(pos) => {
                if !self.pop().unwrap().into_bool() {
                    self.cursor = pos;
                }

                return false;
            }
            Instruction::NewLine => {
                self.push("\n");
                self.flush_print();
            }
            Instruction::Wait => {
                self.push("\n");
                self.flush_print();
                self.printer.wait();
            }
            Instruction::Duplicate => {
                let item = self.peek().copied().unwrap();
                self.push(item);
            }
            Instruction::Nop => {}
            Instruction::Pop => {
                self.pop();
            }
            Instruction::Conditional => {
                let rhs = self.pop().unwrap();
                let lhs = self.pop().unwrap();
                let cond = self.pop_bool();

                self.push(if cond { lhs } else { rhs });
            }
        }

        true
    }

    pub fn run_builtin(&mut self, name: &str) {
        self.builtin[name](self);
    }

    pub fn run(mut self) {
        while let Some(&instruction) = self.instructions.get(self.cursor) {
            if self.run_instruction(instruction) {
                self.cursor += 1;
            }
        }

        self.flush_print();
        self.cursor = 0;
    }
}