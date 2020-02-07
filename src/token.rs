#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimpleOperatorToken {
    /// +
    Add,
    /// -
    Sub,
    /// /
    Div,
    /// *
    Mul,
    /// %
    Rem,
    /// &
    And,
    /// |
    Or,
    /// ^
    Xor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BooleanOperatorToken {
    /// ==
    Equal,
    /// <>
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
    StrLit(&'s str),
    IntLit(u32),
    Builtin(&'s str),
    Variable(&'s str),
    /// ~
    Not,
    SimpleOperator(SimpleOperatorToken),
    BooleanOperator(BooleanOperatorToken),
    AssignOperator(Option<SimpleOperatorToken>),
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// ?
    Question,
    /// #
    Sharp,
}
