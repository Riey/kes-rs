use crate::interner::Symbol;
use crate::operator::{BinaryOperator, TernaryOperator, UnaryOperator};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token<'a> {
    /// 만약
    If,
    /// 혹은
    ElseIf,
    /// 그외
    Else,
    /// 종료
    Exit,
    /// 반복
    While,
    /// ''
    StrLit(Symbol),
    /// 123
    IntLit(u32),
    /// ABC
    Builtin(Symbol),
    /// $ABC
    Variable(Symbol),

    /// !
    UnaryOp(UnaryOperator),

    /// &
    BinaryOp(BinaryOperator),

    /// ? :
    TernaryOp(TernaryOperator, bool),

    /// \# Comment
    Comment(&'a str),

    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// (
    OpenParan,
    /// )
    CloseParan,

    /// @
    Print,
    /// @!
    PrintWait,
    /// @@
    PrintLine,

    /// ;
    SemiColon,

    /// ,
    Comma,

    /// =
    Assign,
}
