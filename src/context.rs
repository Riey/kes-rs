use crate::instruction::Instruction;
use crate::operator::Operator;
use bumpalo::collections::Vec;
use bumpalo::Bump;
use std::collections::HashMap;
use std::convert::TryFrom;

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

pub struct Context<'b> {
    builtin: &'b HashMap<&'b str, fn(&mut Context<'b>)>,
    cursor: usize,
    instructions: &'b [Instruction<'b>],
    stack: Vec<'b, Value<'b>>,
}

impl<'b> Context<'b> {
    pub fn new(
        bump: &'b Bump,
        builtin: &'b HashMap<&'b str, fn(&mut Context<'b>)>,
        instructions: &'b [Instruction<'b>],
    ) -> Self {
        Self {
            builtin,
            cursor: 0,
            instructions,
            stack: Vec::with_capacity_in(100, bump),
        }
    }

    pub fn push(&mut self, v: impl Into<Value<'b>>) {
        self.stack.push(v.into());
    }

    pub fn pop(&mut self) -> Option<Value<'b>> {
        self.stack.pop()
    }

    pub fn run_operator(&mut self, op: Operator) {
        macro_rules! binop {
            ($op:tt) => {
                u32::try_from(self.pop().unwrap()).unwrap() $op u32::try_from(self.pop().unwrap()).unwrap()
            };
        }

        macro_rules! binop_bool {
            ($op:tt) => {
                if binop!($op) {
                    1
                } else {
                    0
                }
            };
        }

        macro_rules! binop_raw_bool {
            ($op:tt) => {
                if self.pop().unwrap() $op self.pop().unwrap() {
                    1
                } else {
                    0
                }
            }
        }

        let num = match op {
            Operator::Equal => binop_raw_bool!(==),
            Operator::NotEqual => binop_raw_bool!(!=),
            Operator::Greater => binop_bool!(>),
            Operator::Less => binop_bool!(<),
            Operator::GreaterOrEqual => binop_bool!(>=),
            Operator::LessOrEqual => binop_bool!(<=),
            Operator::Not => {
                let b: bool = self.pop().unwrap().into();

                if b {
                    1
                } else {
                    0
                }
            }
            Operator::Add => binop!(+),
            Operator::Sub => binop!(-),
            Operator::Mul => binop!(*),
            Operator::Div => binop!(/),
            Operator::Rem => binop!(%),
            Operator::And => binop!(&),
            Operator::Or => binop!(|),
            Operator::Xor => binop!(^),
        };

        self.push(num);
    }

    pub fn run_instruction(&mut self, inst: Instruction<'b>) -> bool {
        match inst {
            Instruction::LoadInt(num) => self.push(num),
            Instruction::LoadStr(str) => self.push(str),
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

            _ => todo!(),
        }

        true
    }

    pub fn run_builtin(&mut self, name: &str) {
        self.builtin[name](self);
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
        self.stack.clear();
    }

    pub fn run(&mut self) {
        while let Some(&instruction) = self.instructions.get(self.cursor) {
            if self.run_instruction(instruction) {
                self.cursor += 1;
            }
        }
    }
}

#[test]
fn run_test() {
    let bump = Bump::with_capacity(8196);
    let builtin = HashMap::default();
    let instructions = crate::parser::parse(
        &bump,
        crate::lexer::lex(
            "
'1 + 2 = ' 1 2 +
",
        ),
    );

    let mut ctx = Context::new(&bump, &builtin, &instructions);

    ctx.run();

    assert_eq!(ctx.stack, &[Value::Str("1 + 2 = "), Value::Int(3)]);
}
