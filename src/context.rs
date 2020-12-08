use crate::builtin::Builtin;
use crate::error::{RuntimeError, RuntimeResult};
use crate::instruction::Instruction;
use crate::instruction::InstructionWithDebug;
use crate::interner::Symbol;
use crate::location::Location;
use crate::operator::{BinaryOperator, TernaryOperator};
use crate::program::Program;
use crate::value::{Value, ValueConvertError};
use ahash::AHashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;

static_assertions::assert_impl_all!(Context: Send, Sync);

pub struct Context<'c> {
    program: &'c Program,
    stack: Vec<Value>,
    pub variables: AHashMap<Symbol, Value>,
    cursor: usize,
}

impl<'c> Context<'c> {
    pub fn new(program: &'c Program) -> Self {
        Self {
            program,
            stack: Vec::with_capacity(50),
            variables: AHashMap::new(),
            cursor: 0,
        }
    }

    pub fn args(&self) -> &[Value] {
        &self.stack[..]
    }

    #[inline]
    fn push(&mut self, v: impl Into<Value>) {
        self.stack.push(v.into());
    }

    #[inline]
    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    #[inline]
    pub fn pop_into<T: TryFrom<Value>>(&mut self) -> T
    where
        T::Error: std::fmt::Debug,
    {
        self.stack.pop().unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn pop_into_ret<T: TryFrom<Value, Error = ValueConvertError>>(
        &mut self,
    ) -> RuntimeResult<T> {
        self.pop_ret()?
            .try_into()
            .map_err(|err: ValueConvertError| {
                RuntimeError::TypeError(err.0, self.current_instruction_location().line)
            })
    }

    #[inline]
    pub fn peek(&mut self) -> Option<&mut Value> {
        self.stack.last_mut()
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
    pub fn pop_str(&mut self) -> String {
        self.pop_into()
    }

    pub fn run_bin_operator(&mut self, op: BinaryOperator) -> RuntimeResult<()> {
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
            BinaryOperator::Equal => {
                binop_raw_bool!(==);
            }
            BinaryOperator::NotEqual => {
                binop_raw_bool!(!=);
            }
            BinaryOperator::Greater => {
                binop_raw_bool!(>);
            }
            BinaryOperator::Less => {
                binop_raw_bool!(<);
            }
            BinaryOperator::GreaterOrEqual => {
                binop_raw_bool!(>=);
            }
            BinaryOperator::LessOrEqual => {
                binop_raw_bool!(<=);
            }
            BinaryOperator::And => {
                binop_bool!(&);
            }
            BinaryOperator::Or => {
                binop_bool!(|);
            }
            BinaryOperator::Xor => {
                binop_bool!(^);
            }
            BinaryOperator::Add => {
                let rhs = self.pop_ret()?;
                let lhs = self.pop_ret()?;

                self.push(match (lhs, rhs) {
                    (Value::Int(l), Value::Int(r)) => Value::Int(l + r),
                    (Value::Int(l), Value::Str(r)) => {
                        let str = format!("{}{}", l, r);
                        Value::Str(str)
                    }
                    (Value::Str(mut l), Value::Int(r)) => {
                        write!(&mut l, "{}", r).unwrap();
                        Value::Str(l)
                    }
                    (Value::Str(l), Value::Str(r)) => Value::Str(l + &r),
                });
            }
            BinaryOperator::Sub => {
                binop!(-);
            }
            BinaryOperator::Mul => {
                binop!(*);
            }
            BinaryOperator::Div => {
                binop!(/);
            }
            BinaryOperator::Rem => {
                binop!(%);
            }
        }

        Ok(())
    }

    pub fn flush_print<B: Builtin>(&mut self, builtin: &mut B) {
        for v in self.stack.drain(..) {
            builtin.print(v);
        }
    }

    pub fn pop_ret(&mut self) -> RuntimeResult<Value> {
        self.pop().ok_or(self.make_err("인자가 부족합니다"))
    }

    pub fn peek_ret(&mut self) -> RuntimeResult<&mut Value> {
        let err = self.make_err("인자가 없습니다");
        self.peek().ok_or(err)
    }

    fn current_instruction_location(&self) -> Location {
        self.program.instructions()[self.cursor].location
    }

    fn make_err(&self, msg: &'static str) -> RuntimeError {
        RuntimeError::ExecutionError(msg, self.current_instruction_location().line)
    }

    pub async fn run_instruction<B: Builtin>(
        &mut self,
        builtin: &mut B,
        inst: InstructionWithDebug,
    ) -> RuntimeResult<()> {
        match inst.inst {
            Instruction::Exit => {
                self.cursor = self.program.instructions().len();
                return Ok(());
            }
            Instruction::LoadInt(num) => self.push(num),
            Instruction::LoadStr(str) => self.push(self.program.resolve(str).unwrap()),
            Instruction::LoadVar(name) => {
                let item = self
                    .variables
                    .get(&name)
                    .ok_or(self.make_err("변수를 찾을수 없습니다"))?
                    .clone();
                self.push(item);
            }
            Instruction::StoreVar(name) => {
                let item = self.pop_ret()?;
                self.variables.insert(name, item);
            }
            Instruction::CallBuiltin(name) => {
                let ret = builtin
                    .run(
                        self.program
                            .resolve(name)
                            .ok_or(self.make_err("알수없는 심볼입니다"))?,
                        self,
                    )
                    .await;
                self.push(ret);
            }
            Instruction::BinaryOperator(op) => self.run_bin_operator(op)?,
            Instruction::UnaryOperator(crate::operator::UnaryOperator::Not) => {
                let v: bool = self.pop_ret()?.into_bool();
                self.push(!v);
            }
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
            Instruction::Print { newline, wait } => {
                self.flush_print(builtin);

                if newline || wait {
                    builtin.new_line();
                }

                if wait {
                    builtin.wait().await;
                }
            }
            Instruction::Duplicate => {
                let item = self.peek_ret()?.clone();
                self.push(item);
            }
            Instruction::Nop => {}
            Instruction::Pop => {
                self.pop();
            }
            Instruction::TernaryOperator(TernaryOperator::Conditional) => {
                let rhs = self.pop_ret()?;
                let lhs = self.pop_ret()?;
                let cond = self.pop_bool();

                self.push(if cond { lhs } else { rhs });
            }
        }

        self.cursor += 1;

        Ok(())
    }

    pub async fn run<B: Builtin>(mut self, mut builtin: B) -> RuntimeResult<()> {
        while let Some(&instruction) = self.program.instructions().get(self.cursor) {
            self.run_instruction(&mut builtin, instruction).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::builtin::RecordBuiltin;
    use crate::error::{RuntimeError, RuntimeResult};
    use crate::program::Program;
    use pretty_assertions::assert_eq;

    fn test_impl(code: &str) -> RuntimeResult<crate::builtin::RecordBuiltin> {
        let program = Program::from_source(code).unwrap();
        let mut builtin = RecordBuiltin::new();
        let ctx = Context::new(&program);

        futures_executor::block_on(ctx.run(&mut builtin))?;

        Ok(builtin)
    }

    #[cfg(test)]
    fn try_test(code: &str, expected: &str) {
        assert_eq!(test_impl(code).unwrap().text(), expected);
    }

    #[test]
    fn error_line_no() {
        let err = test_impl(
            "
    2 + '2';
    # 3번째줄
    1 - '1'; #4번째줄
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
    fn if_test() {
        try_test(
            "만약 !1 { @'2'; } 그외 { @'3'; } 만약 0 { @'4'; } 그외 { @'5'; }",
            "3@5@",
        );
    }

    #[test]
    fn loop_test() {
        try_test(
            "$0 = 1; 반복 $0 < 10 { @@$0; $0 = $0 + 1; } @@$0;",
            "12345678910",
        );
    }
}
