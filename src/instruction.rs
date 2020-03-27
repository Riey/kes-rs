use crate::operator::Operator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction<'s> {
    Nop,
    Exit,
    Pop,
    PopExternal,
    Duplicate,
    Conditional,
    LoadInt(u32),
    LoadStr(&'s str),
    LoadVar(&'s str),
    StoreVar(&'s str),
    LoadBuiltin(&'s str),
    CallBuiltin(&'s str),
    Print,
    PrintLine,
    PrintWait,
    Operator(Operator),
    Goto(u32),
    GotoIfNot(u32),
    StartBlock,
    EndBlock,
}
