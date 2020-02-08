use crate::operator::Operator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction<'s> {
    Nop,
    Pop,
    Duplicate,
    PushInt(u32),
    PushStr(&'s str),
    PushVar(&'s str),
    CallBuiltin(&'s str),
    NewLine,
    Wait,
    Operator(Operator),
    Goto(usize),
    GotoIfNot(usize),
}
