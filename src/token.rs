use crate::operator::{BinaryOperator, TernaryOperator, UnaryOperator};

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
    While,
    /// ''
    StrLit(&'s str),
    /// 123
    IntLit(u32),
    /// ABC
    Builtin(&'s str),
    /// $ABC
    Variable(&'s str),

    /// !
    UnaryOp(UnaryOperator),

    /// &
    BinaryOp(BinaryOperator),

    /// ? :
    TernaryOp(TernaryOperator, bool),

    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// (
    OpenParan,
    /// )
    CloseParan,

    /// @@
    Print,
    /// @!
    PrintWait,
    /// @
    PrintLine,

    /// ;
    SemiColon,

    /// ,
    Comma,

    /// =
    Assign,
}
