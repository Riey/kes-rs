use crate::operator::Operator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token<'s> {
    /// 만약
    If,
    /// 혹은
    ElseIf,
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
    /// (
    OpenParan,
    /// )
    CloseParan,
    /// :
    Colon,
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
    /// _
    Underscore,
}
