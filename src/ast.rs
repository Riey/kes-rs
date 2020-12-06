use crate::operator::BinaryOperator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stmt<'s> {
    Assign {
        var: &'s str,
        value: Expr<'s>,
    },
    Print {
        values: Vec<Expr<'s>>,
        newline: bool,
        wait: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr<'s> {
    Number(u32),
    String(&'s str),
    Variable(&'s str),

    BinaryOp {
        lhs: Box<Expr<'s>>,
        rhs: Box<Expr<'s>>,
        op: BinaryOperator,
    },
}
