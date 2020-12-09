use crate::operator::{BinaryOperator, UnaryOperator};
use crate::{interner::Symbol, location::Location, operator::TernaryOperator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stmt {
    Assign {
        var: Symbol,
        value: Expr,
        location: Location,
    },
    Print {
        values: Vec<Expr>,
        newline: bool,
        wait: bool,
        location: Location,
    },
    If {
        arms: Vec<(Expr, Vec<Stmt>, Location)>,
        other: Vec<Stmt>,
        other_location: Location,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        location: Location,
    },
    Expression {
        expr: Expr,
        location: Location,
    },
    Exit {
        location: Location,
    },
}

impl Stmt {
    pub fn location(&self) -> Location {
        match self {
            Stmt::Assign { location, .. }
            | Stmt::Print { location, .. }
            | Stmt::While { location, .. }
            | Stmt::Expression { location, .. }
            | Stmt::Exit { location } => *location,
            Stmt::If { arms, .. } => arms[0].2,
        }
    }

    pub fn is_block(&self) -> bool {
        match self {
            Stmt::If { .. } | Stmt::While { .. } => true,
            Stmt::Assign { .. }
            | Stmt::Print { .. }
            | Stmt::Expression { .. }
            | Stmt::Exit { .. } => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    Number(u32),
    String(Symbol),
    Variable(Symbol),
    BuiltinFunc {
        name: Symbol,
        args: Vec<Expr>,
    },

    Nop(Box<Expr>),

    UnaryOp {
        value: Box<Expr>,
        op: UnaryOperator,
    },

    BinaryOp {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinaryOperator,
    },

    TernaryOp {
        lhs: Box<Expr>,
        mhs: Box<Expr>,
        rhs: Box<Expr>,
        op: TernaryOperator,
    },
}

impl Expr {
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

    pub fn ternary_op(self, mhs: Self, rhs: Self, op: TernaryOperator) -> Self {
        Expr::TernaryOp {
            lhs: Box::new(self),
            mhs: Box::new(mhs),
            rhs: Box::new(rhs),
            op,
        }
    }
}
