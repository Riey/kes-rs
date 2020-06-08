use crate::operator::Operator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction<'s> {
    Nop,
    Exit,
    Pop,
    PopExternal(u32),
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

#[derive(Clone, Copy, Debug)]
pub struct InstructionWithDebug<'s> {
    pub inst: Instruction<'s>,
    pub line_no: usize,
}

impl<'s> InstructionWithDebug<'s> {
    pub fn new(inst: Instruction<'s>, line_no: usize) -> Self {
        Self { inst, line_no }
    }
}

impl<'s> PartialEq<Instruction<'s>> for InstructionWithDebug<'s> {
    fn eq(&self, other: &Instruction<'s>) -> bool {
        self.inst == *other
    }
}
