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
    /// ~
    Not,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorToken {
    Simple(SimpleOperatorToken),
    Boolean(BooleanOperatorToken),
    Assign(Option<SimpleOperatorToken>),
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
    Operator(OperatorToken),
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// ?
    Question,
    /// :
    Colon,
    /// #
    Sharp,
    /// @
    At,
}
