use crate::builtin::Builtin;
use crate::console::KesConsole;
use crate::instruction::Instruction;
use crate::operator::Operator;
use crate::value::Value;
use ahash::AHashMap;
use arrayvec::ArrayVec;
use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;

pub struct Context<'c> {
    pub bump: &'c Bump,
    pub instructions: &'c [Instruction<'c>],
    pub stack: Vec<'c, Vec<'c, Value<'c>>>,
    pub variables: AHashMap<&'c str, Value<'c>>,
    cursor: usize,
}

impl<'c> Context<'c> {
    pub fn new(bump: &'c Bump, instructions: &'c [Instruction<'c>]) -> Self {
        Self {
            bump,
            instructions,
            stack: Vec::with_capacity_in(50, bump),
            variables: AHashMap::new(),
            cursor: 0,
        }
    }

    #[inline]
    pub fn current_block(&mut self) -> &mut Vec<'c, Value<'c>> {
        self.stack.last_mut().unwrap()
    }

    #[inline]
    pub fn push(&mut self, v: impl Into<Value<'c>>) {
        self.current_block().push(v.into());
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Value<'c>> {
        self.current_block().pop()
    }

    #[inline]
    pub fn pop_into<T: TryFrom<Value<'c>>>(&mut self) -> T
    where
        T::Error: std::fmt::Debug,
    {
        self.current_block().pop().unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn peek(&mut self) -> Option<&mut Value<'c>> {
        self.current_block().last_mut()
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

                self.push(if !b { 1 } else { 0 });
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

    pub fn flush_print<C: KesConsole>(&mut self, console: &mut C) {
        for v in self.current_block().drain(..) {
            console.print(v);
        }
    }

    #[inline]
    pub fn bump(&self) -> &'c Bump {
        self.bump
    }

    pub async fn run_instruction<B: Builtin, C: KesConsole>(&mut self, builtin: &mut B, console: &mut C, inst: Instruction<'c>) {
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
            Instruction::LoadBuiltin(name) => {
                self.push(builtin.load(name, self.bump));
            }
            Instruction::CallBuiltin(name) => {
                let ret = builtin.run(name, self, console).await;
                self.stack.pop();
                if let Some(ret) = ret {
                    self.push(ret);
                }
            }
            Instruction::Operator(op) => self.run_operator(op),
            Instruction::Goto(pos) => {
                self.cursor = pos as usize;
                return;
            }
            Instruction::GotoIfNot(pos) => {
                if !self.pop().unwrap().into_bool() {
                    self.cursor = pos as usize;
                    return;
                }
            }
            Instruction::StartBlock => {
                self.stack.push(Vec::with_capacity_in(10, self.bump()));
            }
            Instruction::EndBlock => {
                self.stack.pop();
            }
            Instruction::Print => {
                self.flush_print(console);
            }
            Instruction::PrintLine => {
                self.flush_print(console);
                console.new_line();
            }
            Instruction::PrintWait => {
                self.flush_print(console);
                console.new_line();
                console.wait().await;
            }
            Instruction::Duplicate => {
                let item = self.peek().copied().unwrap();
                self.push(item);
            }
            Instruction::Nop => {}
            Instruction::Pop => {
                self.pop();
            }
            Instruction::PopExternal(num) => {
                let start = if num == 0 {
                    0
                } else {
                    self.current_block().len() - num as usize
                };

                let buf: ArrayVec<[_; 20]> = self.current_block().drain(start..).collect();
                let pos = self.stack.len() - 2;
                self.stack[pos].extend(buf);
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

    pub async fn run<B: Builtin, C: KesConsole>(mut self, mut builtin: B, mut console: C) {
        while let Some(&instruction) = self.instructions.get(self.cursor) {
            self.run_instruction(&mut builtin, &mut console, instruction).await;
        }
    }
}

#[cfg(test)]
fn try_test(code: &str, expected_builtin: &str, expected_console: &str) {
    use crate::builtin::RecordBuiltin;
    use crate::console::RecordConsole;
    use crate::parser::parse;
    use pretty_assertions::assert_eq;

    let bump = Bump::with_capacity(8196);
    let instructions = parse(&bump, code).unwrap();

    let mut builtin = RecordBuiltin::new();
    let mut console = RecordConsole::new();
    let ctx = Context::new(&bump, &instructions);

    futures::executor::block_on(ctx.run(&mut builtin, &mut console));

    assert_eq!(builtin.0, expected_builtin);
    assert_eq!(console.0, expected_console);
}

#[test]
fn pop_external_test() {
    try_test(
        "
만약 1 2 ~= {
    2 3 [!]
    4 5 [!1] #
}
@
",
        "",
        "4@#235@",
    );
}

#[test]
fn str_select_test() {
    try_test(
        "
선택 '2' {
    '1' {
        3:
    }
    '2' {
        4:
    }
    _ {
        5:
    }
}
",
        "",
        "4",
    );
}

#[test]
fn if_test() {
    try_test(
        "만약 1 ~ { '2'@ } 그외 { '3'@ } 만약 0 { '3'@ } 그외 {  '4'@ }",
        "",
        "3@4@",
    );
}

#[test]
fn loop_test() {
    try_test("1 [$0] 반복 $0 10 < { $0: $0 1 + [$0] } $0:", "", "12345678910");
}

#[test]
fn complex_test() {
    try_test("1 2 + [$0] 만약 $0 3 = { 선택 $0 { 1 { 2: } 3 { '?': } _ { '1': } } } 그외 { 만약 3 { 4: } 만약 '1' { 1: } } $0:", "", "?3");
}
