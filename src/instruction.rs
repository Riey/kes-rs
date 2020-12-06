use crate::{
    location::Location,
    operator::{BinaryOperator, UnaryOperator},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Instruction<'s> {
    Nop,
    Exit,
    Pop,
    Duplicate,
    Conditional,
    LoadInt(u32),
    LoadStr(&'s str),
    LoadVar(&'s str),
    StoreVar(&'s str),
    LoadBuiltin(&'s str),
    CallBuiltin(&'s str),
    Print { newline: bool, wait: bool },
    BinaryOperator(BinaryOperator),
    UnaryOperator(UnaryOperator),
    Goto(u32),
    GotoIfNot(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InstructionWithDebug<'s> {
    pub inst: Instruction<'s>,
    pub location: Location,
}
