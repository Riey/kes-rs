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
            parse("@ '123' 123").unwrap(),
            [
                Stmt::Print { values: vec![Expr::String("123"), Expr::Number(123)], newline: true, wait: false }
            ]
        )
    }
}
