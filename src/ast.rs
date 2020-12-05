use crate::operator::BinaryOperator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stmt<'s> {
    Assign { var: &'s str, value: Expr<'s> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr<'s> {
    Number(u32),
    String(&'s str),

    BinaryOp {
        lhs: Box<Expr<'s>>,
        rhs: Box<Expr<'s>>,
        op: BinaryOperator,
    },
}
