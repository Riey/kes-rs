#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrLitPostFix {
    L,
    W,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorToken {
    /// +
    Add,
    /// +=
    AddAssign,
    /// -
    Sub,
    /// -=
    SubAssign,
    /// /
    Div,
    /// /=
    DivAssign,
    /// *
    Mul,
    /// *=
    MulAssign,
    /// %
    Rem,
    /// %=
    RemAssign,
    /// &
    And,
    /// &=
    AndAssign,
    /// |
    Or,
    /// |=
    OrAssign,
    /// ^
    Xor,
    /// ^=
    XorAssign,
    /// !
    Not,
    /// =
    Assign,
    /// ==
    Equal,
    /// !=
    NotEqual,
    /// <
    Less,
    /// <=
    LessOrEqual,
    /// >
    Greater,
    /// >=
    GreaterOrEqual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token<'s> {
    /// 그외
    Else,
    /// 선택
    Select,
    StrLit(&'s str, Option<StrLitPostFix>),
    IntLit(u32),
    Builtin(&'s str),
    Variable(&'s str),
    Operator(OperatorToken),
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Comma,
    Question,
    Sharp,
}
