use crate::builtin::Builtin;
use crate::instruction::Instruction;
use crate::operator::Operator;
use crate::value::Value;
use ahash::AHashMap;
use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;

pub struct Context<'c> {
    pub bump: &'c Bump,
    pub instructions: &'c [Instruction<'c>],
    pub stack: Vec<'c, Value<'c>>,
    pub variables: AHashMap<&'c str, Value<'c>>,
    mark: Vec<'c, usize>,
    cursor: usize,
}

impl<'c> Context<'c> {
    pub fn new(bump: &'c Bump, instructions: &'c [Instruction<'c>]) -> Self {
        Self {
            bump,
            instructions,
            stack: Vec::with_capacity_in(50, bump),
            variables: AHashMap::new(),
            mark: Vec::with_capacity_in(10, bump),
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
                binop_raw_bool!(>);
            }
            Operator::Less => {
                binop_raw_bool!(<);
            }
            Operator::GreaterOrEqual => {
                binop_raw_bool!(>=);
            }
            Operator::LessOrEqual => {
                binop_raw_bool!(<=);
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

    pub fn flush_print<B: Builtin>(&mut self, builtin: &mut B) {
        for v in self.stack.drain(..) {
            builtin.print(v);
        }
    }

    #[inline(always)]
    pub fn bump(&self) -> &'c Bump {
        self.bump
    }

    pub fn run_instruction<B: Builtin>(&mut self, builtin: &mut B, inst: Instruction<'c>) {
        match inst {
            Instruction::Exit => {
                self.cursor = self.instructions.len();
                return;
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
                return;
            }
            Instruction::GotoIfNot(pos) => {
                if !self.pop().unwrap().into_bool() {
                    self.cursor = pos;
                    return;
                }
            }
            Instruction::MarkStack => {
                self.mark.push(self.stack.len() - 1);
            }
            Instruction::RemoveMarked => {
                self.stack.remove(self.mark.pop().unwrap());
            }
            Instruction::NewLine => {
                self.flush_print(builtin);
                builtin.new_line();
            }
            Instruction::Wait => {
                self.flush_print(builtin);
                builtin.new_line();
                builtin.wait();
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

        self.cursor += 1;
    }

    pub fn run<B: Builtin>(mut self, mut builtin: B) {
        while let Some(&instruction) = self.instructions.get(self.cursor) {
            self.run_instruction(&mut builtin, instruction);
        }

        self.flush_print(&mut builtin);
    }
}

#[test]
fn str_select_test() {
    use crate::builtin::RecordBuiltin;
    use crate::parser::parse;
    let bump = Bump::with_capacity(8196);
    let instructions = parse(
        &bump,
        "
선택 '2' {
    '1' {
        3
    }
    '2' {
        4
    }
    그외 {
        5
    }
}
",
    );

    let mut builtin = RecordBuiltin::new();
    let ctx = Context::new(&bump, &instructions);

    ctx.run(&mut builtin);

    assert_eq!(builtin.text(), "4");
}
