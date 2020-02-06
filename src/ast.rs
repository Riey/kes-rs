#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Eq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
    Var(String),
    Builtin(String, Vec<Expr>),
}
