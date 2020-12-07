use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// !
    Not,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TernaryOperator {
    /// ? :
    Conditional,
}
