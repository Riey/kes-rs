use crate::operator::Operator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token<'s> {
    /// 그외
    Else,
    /// 선택
    Select,
    /// 종료
    Exit,
    /// 반복
    Loop,
    /// ''
    StrLit(&'s str),
    /// 123
    IntLit(u32),
    /// ABC
    Builtin(&'s str),
    /// $ABC
    Variable(&'s str),
    /// +-/*
    Operator(Operator),
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// #
    Sharp,
    /// @
    At,
    /// \[$<>]
    Assign(&'s str),
    /// \[+]
    Duplicate,
    /// \[-]
    Pop,
    /// \[?]
    Conditional,
}
