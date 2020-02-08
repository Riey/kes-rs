use crate::operator::Operator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token<'s> {
    /// 그외
    Else,
    /// 선택
    Select,
    StrLit(&'s str),
    IntLit(u32),
    Builtin(&'s str),
    Variable(&'s str),
    Operator(Operator),
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// ?
    Question,
    /// #
    Sharp,
    /// @
    At,
    /// ->
    Assign,
}
