use crate::location::Location;
use crate::operator::{BinaryOperator, UnaryOperator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stmt<'s> {
    Assign {
        var: &'s str,
        value: Expr<'s>,
        location: Location,
    },
    Print {
        values: Vec<Expr<'s>>,
        newline: bool,
        wait: bool,
        location: Location,
    },
    If {
        arms: Vec<(Expr<'s>, Vec<Stmt<'s>>)>,
        other: Vec<Stmt<'s>>,
        location: Location,
    },
    While {
        cond: Expr<'s>,
        body: Vec<Stmt<'s>>,
        location: Location,
    },
    Expression {
        expr: Expr<'s>,
        location: Location,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr<'s> {
    Number(u32),
    String(&'s str),
    Variable(&'s str),

    UnaryOp {
        value: Box<Expr<'s>>,
        op: UnaryOperator,
    },

    BinaryOp {
        lhs: Box<Expr<'s>>,
        rhs: Box<Expr<'s>>,
        op: BinaryOperator,
    },
}

impl<'s> Expr<'s> {
    pub fn unary_op(self, op: UnaryOperator) -> Self {
        Expr::UnaryOp {
            value: Box::new(self),
            op,
        }
    }

    pub fn binary_op(self, rhs: Self, op: BinaryOperator) -> Self {
        Expr::BinaryOp {
            lhs: Box::new(self),
            rhs: Box::new(rhs),
            op,
        }
    }
}
