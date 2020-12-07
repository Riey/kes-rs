use crate::{
    interner::Symbol,
    location::Location,
    operator::{BinaryOperator, TernaryOperator, UnaryOperator},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    Nop,
    Exit,
    Pop,
    Duplicate,
    LoadInt(u32),
    LoadStr(Symbol),
    LoadVar(Symbol),
    StoreVar(Symbol),
    CallBuiltin(Symbol),
    Print { newline: bool, wait: bool },
    BinaryOperator(BinaryOperator),
    UnaryOperator(UnaryOperator),
    TernaryOperator(TernaryOperator),
    Goto(u32),
    GotoIfNot(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InstructionWithDebug {
    pub inst: Instruction,
    pub location: Location,
}
