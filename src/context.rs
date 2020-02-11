use crate::builtin::Builtin;
use crate::instruction::Instruction;
use crate::operator::Operator;
use crate::printer::Printer;
use ahash::AHashMap;
use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
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
    #[inline]
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

    #[inline]
    fn try_from(v: Value<'b>) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n),
            _ => Err(ValueConvertError),
        }
    }
}

impl<'b> TryFrom<Value<'b>> for &'b str {
    type Error = ValueConvertError;

    #[inline]
    fn try_from(v: Value<'b>) -> Result<Self, Self::Error> {
        match v {
            Value::Str(s) => Ok(s),
            _ => Err(ValueConvertError),
        }
    }
}

pub struct Context<'b: 'c, 'c, P: Printer> {
    pub bump: &'c Bump,
    pub instructions: &'c [Instruction<'c>],
    pub printer: P,
    pub print_buffer: &'c mut String<'b>,
    pub stack: Vec<'c, Value<'c>>,
    pub variables: AHashMap<&'c str, Value<'c>>,
    cursor: usize,
}

impl<'b: 'c, 'c, P: Printer> Context<'b, 'c, P> {
    pub fn new(
        bump: &'c Bump,
        instructions: &'c [Instruction<'b>],
        printer: P,
        print_buffer: &'c mut String<'b>,
    ) -> Self {
        Self {
            bump,
            instructions,
            stack: Vec::with_capacity_in(50, bump),
            printer,
            print_buffer,
            variables: AHashMap::new(),
            cursor: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, v: impl Into<Value<'c>>) {
        self.stack.push(v.into());
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Value<'c>> {
        self.stack.pop()
    }

    #[inline]
    pub fn pop_into<T: TryFrom<Value<'c>>>(&mut self) -> T
    where
        T::Error: std::fmt::Debug,
    {
        self.pop().unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn peek(&self) -> Option<&Value<'c>> {
        self.stack.last()
    }

    #[inline]
    pub fn pop_u32(&mut self) -> u32 {
        self.pop_into()
    }

    #[inline]
    pub fn pop_bool(&mut self) -> bool {
        self.pop_into()
    }

    #[inline]
    pub fn pop_str(&mut self) -> &'c str {
        self.pop_into()
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
                let rhs = self.pop_bool();
                let lhs = self.pop_bool();
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
            Operator::And => {
                binop_bool!(&);
            }
            Operator::Or => {
                binop_bool!(|);
            }
            Operator::Xor => {
                binop_bool!(^);
            }
            Operator::Not => {
                let b = self.pop_bool();

                self.push(if b { 1 } else { 0 });
            }
            Operator::Add => {
                let rhs = self.pop().unwrap();
                let lhs = self.pop().unwrap();

                self.push(match (lhs, rhs) {
                    (Value::Int(l), Value::Int(r)) => Value::Int(l + r),
                    (Value::Int(l), Value::Str(r)) => {
                        let mut buf = String::with_capacity_in(r.len() + 10, self.bump);
                        write!(&mut buf, "{}", l).unwrap();
                        buf.push_str(r);
                        Value::Str(buf.into_bump_str())
                    }
                    (Value::Str(l), Value::Int(r)) => {
                        let mut buf = String::with_capacity_in(l.len() + 10, self.bump);
                        buf.push_str(l);
                        write!(&mut buf, "{}", r).unwrap();
                        Value::Str(buf.into_bump_str())
                    }
                    (Value::Str(l), Value::Str(r)) => {
                        let mut buf = String::with_capacity_in(l.len() + r.len(), self.bump);
                        buf.push_str(l);
                        buf.push_str(r);
                        Value::Str(buf.into_bump_str())
                    }
                });
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

    pub fn run_instruction<B: Builtin>(&mut self, builtin: &mut B, inst: Instruction<'c>) -> bool {
        match inst {
            Instruction::Exit => {
                self.cursor = self.instructions.len();
                return false;
            }
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
            Instruction::CallBuiltin(name) => builtin.run(name, self),
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

    pub fn run<B: Builtin>(mut self, mut builtin: B) {
        while let Some(&instruction) = self.instructions.get(self.cursor) {
            if self.run_instruction(&mut builtin, instruction) {
                self.cursor += 1;
            }
        }

        self.flush_print();
        self.cursor = 0;
    }
}
