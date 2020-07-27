use crate::builtin::Builtin;
use crate::error::{RuntimeError, RuntimeResult};
use crate::instruction::Instruction;
use crate::instruction::InstructionWithDebug;
use crate::operator::Operator;
use crate::value::Value;
use crate::value::ValueConvertError;
use ahash::AHashMap;
use arrayvec::ArrayVec;
use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;

pub struct Context<'c> {
    pub bump: &'c Bump,
    pub instructions: &'c [InstructionWithDebug<'c>],
    pub stack: Vec<'c, Vec<'c, Value<'c>>>,
    pub variables: AHashMap<&'c str, Value<'c>>,
    cursor: usize,
}

impl<'c> Context<'c> {
    pub fn new(bump: &'c Bump, instructions: &'c [InstructionWithDebug<'c>]) -> Self {
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
    pub fn pop_into_ret<T: TryFrom<Value<'c>, Error = ValueConvertError>>(
        &mut self,
    ) -> RuntimeResult<T> {
        self.pop_ret()?
            .try_into()
            .map_err(|err: ValueConvertError| {
                RuntimeError::TypeError(err.0, self.current_instruction_line_no())
            })
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

    pub fn run_operator(&mut self, op: Operator) -> RuntimeResult<()> {
        macro_rules! binop {
            ($op:tt) => {
                let rhs: u32 = self.pop_into_ret()?;
                let lhs: u32 = self.pop_into_ret()?;
                self.push(lhs $op rhs);
            };
        }

        macro_rules! binop_bool {
            ($op:tt) => {
                let rhs = self.pop_ret()?.into_bool();
                let lhs = self.pop_ret()?.into_bool();
                self.push(if lhs $op rhs {
                    1
                } else {
                    0
                });
            };
        }

        macro_rules! binop_raw_bool {
            ($op:tt) => {
                let rhs = self.pop_ret()?;
                let lhs = self.pop_ret()?;
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
                let b = self.pop_ret()?.into_bool();

                self.push(if !b { 1 } else { 0 });
            }
            Operator::Add => {
                let rhs = self.pop_ret()?;
                let lhs = self.pop_ret()?;

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

        Ok(())
    }

    pub fn flush_print<B: Builtin>(&mut self, builtin: &mut B) {
        for v in self.current_block().drain(..) {
            builtin.print(v);
        }
    }

    pub fn pop_ret(&mut self) -> RuntimeResult<Value<'c>> {
        self.pop().ok_or(self.make_err("인자가 부족합니다"))
    }

    pub fn peek_ret(&mut self) -> RuntimeResult<&mut Value<'c>> {
        let err = self.make_err("인자가 없습니다");
        self.peek().ok_or(err)
    }

    fn current_instruction_line_no(&self) -> usize {
        self.instructions[self.cursor].line_no
    }

    fn make_err(&self, msg: &'static str) -> RuntimeError {
        RuntimeError::ExecutionError(msg, self.current_instruction_line_no())
    }

    #[inline]
    pub fn bump(&self) -> &'c Bump {
        self.bump
    }

    pub fn run_instruction<B: Builtin>(
        &mut self,
        builtin: &mut B,
        inst: InstructionWithDebug<'c>,
    ) -> RuntimeResult<()> {
        match inst.inst {
            Instruction::Exit => {
                self.cursor = self.instructions.len();
                return Ok(());
            }
            Instruction::LoadInt(num) => self.push(num),
            Instruction::LoadStr(str) => self.push(str),
            Instruction::LoadVar(name) => {
                let item = self.variables[name];
                self.push(item);
            }
            Instruction::StoreVar(name) => {
                let item = self.pop_ret()?;
                self.variables.insert(name, item);
            }
            Instruction::LoadBuiltin(name) => {
                self.push(builtin.load(name, self.bump));
            }
            Instruction::CallBuiltin(name) => {
                let ret = builtin.run(name, self);
                self.stack.pop();
                if let Some(ret) = ret {
                    self.push(ret);
                }
            }
            Instruction::Operator(op) => self.run_operator(op)?,
            Instruction::Goto(pos) => {
                self.cursor = pos as usize;
                return Ok(());
            }
            Instruction::GotoIfNot(pos) => {
                if !self.pop_ret()?.into_bool() {
                    self.cursor = pos as usize;
                    return Ok(());
                }
            }
            Instruction::StartBlock => {
                self.stack.push(Vec::with_capacity_in(10, self.bump()));
            }
            Instruction::EndBlock => {
                self.stack.pop();
            }
            Instruction::Print => {
                self.flush_print(builtin);
            }
            Instruction::PrintLine => {
                self.flush_print(builtin);
                builtin.new_line();
            }
            Instruction::PrintWait => {
                self.flush_print(builtin);
                builtin.new_line();
                builtin.wait();
            }
            Instruction::Duplicate => {
                let item = *self.peek_ret()?;
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
                let rhs = self.pop_ret()?;
                let lhs = self.pop_ret()?;
                let cond = self.pop_bool();

                self.push(if cond { lhs } else { rhs });
            }
        }

        self.cursor += 1;

        Ok(())
    }

    pub fn run<B: Builtin>(mut self, mut builtin: B) -> RuntimeResult<()> {
        while let Some(&instruction) = self.instructions.get(self.cursor) {
            self.run_instruction(&mut builtin, instruction)?;
        }

        Ok(())
    }
}

#[cfg(test)]
fn test_impl(code: &str) -> RuntimeResult<crate::builtin::RecordBuiltin> {
    use crate::builtin::RecordBuiltin;
    use crate::parser::parse;

    let bump = Bump::with_capacity(8196);
    let instructions = parse(&bump, code).unwrap();

    let mut builtin = RecordBuiltin::new();
    let ctx = Context::new(&bump, &instructions);

    ctx.run(&mut builtin)?;

    Ok(builtin)
}

#[cfg(test)]
fn try_test(code: &str, expected: &str) {
    use pretty_assertions::assert_eq;

    assert_eq!(test_impl(code).unwrap().text(), expected);
}

#[test]
fn error_line_no() {
    let err = test_impl(
        "
    2 '2' +
    ; 3번째줄
    1 '1' - ;4번째줄
    ",
    )
    .err()
    .unwrap();

    match err {
        RuntimeError::TypeError("str", 4) => {}
        _ => panic!("unexpected error"),
    }
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
        "4",
    );
}

#[test]
fn if_test() {
    try_test(
        "만약 1 ~ { '2'@ } 그외 { '3'@ } 만약 0 { '3'@ } 그외 {  '4'@ }",
        "3@4@",
    );
}

#[test]
fn loop_test() {
    try_test("1 [$0] 반복 $0 10 < { $0: $0 1 + [$0] } $0:", "12345678910");
}

#[test]
fn complex_test() {
    try_test("1 2 + [$0] 만약 $0 3 = { 선택 $0 { 1 { 2: } 3 { '?': } _ { '1': } } } 그외 { 만약 3 { 4: } 만약 '1' { 1: } } $0:", "?3");
}
