#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimpleOperator {
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
pub enum BooleanOperator {
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
pub enum Operator {
    Simple(SimpleOperator),
    Boolean(BooleanOperator),
    Assign(Option<SimpleOperator>),
}
