use crate::operator::Operator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction<'s> {
    Nop,
    Pop,
    Duplicate,
    Conditional,
    LoadInt(u32),
    LoadStr(&'s str),
    LoadVar(&'s str),
    StoreVar(&'s str),
    CallBuiltin(&'s str),
    NewLine,
    Wait,
    Operator(Operator),
    Goto(usize),
    GotoIfNot(usize),
}
