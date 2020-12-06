use crate::ast::Stmt;
use crate::error::ParseError;
use crate::lexer::Lexer;

pub fn parse(s: &str) -> Result<Vec<Stmt>, ParseError> {
    let lexer = Lexer::new(s);
    crate::grammar::ProgramParser::new().parse(lexer)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::{
        ast::{Expr, Stmt},
        operator::BinaryOperator,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn add() {
        assert_eq!(
            parse("$1 = 1 + 2 * 3;").unwrap(),
            [Stmt::Assign {
                var: "1",
                value: Expr::BinaryOp {
                    lhs: Box::new(Expr::Number(1)),
                    rhs: Box::new(Expr::BinaryOp {
                        lhs: Box::new(Expr::Number(2)),
                        rhs: Box::new(Expr::Number(3)),
                        op: BinaryOperator::Mul,
                    }),
                    op: BinaryOperator::Add,
                }
            }]
        );
    }

    #[test]
    fn print() {
        assert_eq!(
            parse("@ '123' 123;").unwrap(),
            [Stmt::Print {
                values: vec![Expr::String("123"), Expr::Number(123)],
                newline: true,
                wait: false
            }]
        )
    }

    #[test]
    fn variable() {
        assert_eq!(
            parse(
                "
            $1 = 1;
            $2 = 2;
            $3 = $1 + $2;
            "
            )
            .unwrap(),
            [
                Stmt::Assign {
                    var: "1",
                    value: Expr::Number(1)
                },
                Stmt::Assign {
                    var: "2",
                    value: Expr::Number(2)
                },
                Stmt::Assign {
                    var: "3",
                    value: Expr::BinaryOp {
                        lhs: Box::new(Expr::Variable("1")),
                        rhs: Box::new(Expr::Variable("2")),
                        op: BinaryOperator::Add,
                    }
                },
            ]
        );
    }

    #[test]
    fn compare() {
        assert_eq!(
            parse("1 > 2;").unwrap(),
            [Stmt::Expression(
                Expr::Number(1).binary_op(Expr::Number(2), BinaryOperator::Greater)
            )]
        )
    }

    #[test]
    fn if_simple() {
        assert_eq!(
            parse(
                "
        만약 1 + 2 > 2 {
            @ '1 + 2는 2보다 크다';
        }
        "
            )
            .unwrap(),
            [Stmt::If {
                arms: vec![(
                    Expr::Number(1)
                        .binary_op(Expr::Number(2), BinaryOperator::Add)
                        .binary_op(Expr::Number(2), BinaryOperator::Greater),
                    vec![Stmt::Print {
                        values: vec![Expr::String("1 + 2는 2보다 크다"),],
                        newline: true,
                        wait: false,
                    }]
                )],
                other: vec![],
            }]
        )
    }

    #[test]
    fn if_else_simple() {
        assert_eq!(
            parse(
                "
        만약 1 + 2 > 2 {
            @ '1 + 2는 2보다 크다';
        } 혹은 1 + 2 == 2 {
            @ '1 + 2는 2다';
        } 그외 {
            @ '1 + 2는 2보다 작다';
        }
        "
            )
            .unwrap(),
            [Stmt::If {
                arms: vec![
                    (
                        Expr::Number(1)
                            .binary_op(Expr::Number(2), BinaryOperator::Add)
                            .binary_op(Expr::Number(2), BinaryOperator::Greater),
                        vec![Stmt::Print {
                            values: vec![Expr::String("1 + 2는 2보다 크다"),],
                            newline: true,
                            wait: false,
                        }]
                    ),
                    (
                        Expr::Number(1)
                            .binary_op(Expr::Number(2), BinaryOperator::Add)
                            .binary_op(Expr::Number(2), BinaryOperator::Equal),
                        vec![Stmt::Print {
                            values: vec![Expr::String("1 + 2는 2다"),],
                            newline: true,
                            wait: false,
                        }]
                    ),
                ],
                other: vec![Stmt::Print {
                    values: vec![Expr::String("1 + 2는 2보다 작다"),],
                    newline: true,
                    wait: false,
                }],
            }]
        )
    }

    #[test]
    fn loop_test() {
        assert_eq!(
            parse("반복 2 > 1 { @123; }").unwrap(),
            [Stmt::While {
                cond: Expr::Number(2).binary_op(Expr::Number(1), BinaryOperator::Greater),
                body: vec![Stmt::Print {
                    values: vec![Expr::Number(123)],
                    newline: true,
                    wait: false
                },],
            }]
        );
    }
}
