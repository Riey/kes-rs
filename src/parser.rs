use crate::ast::Stmt;
use crate::error::ParseError;
use crate::interner::Interner;
use crate::lexer::Lexer;

pub fn parse<'s>(s: &'s str, interner: &mut Interner) -> Result<Vec<Stmt<'s>>, ParseError<'s>> {
    let lexer = Lexer::new(s, interner);
    crate::grammar::ProgramParser::new().parse(lexer)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::{
        ast::{Expr, Stmt},
        interner::Interner,
        location::Location,
        operator::BinaryOperator,
    };
    use pretty_assertions::assert_eq;

    macro_rules! make_test {
        ($name:ident, $code:expr, [$(($sym:ident, $text:expr),)*], $program:expr) => {
            #[test]
            fn $name() {
                let mut interner = Interner::new();

                let program = match parse($code, &mut interner) {
                    Ok(program) => program,
                    Err(err) => {
                        panic!("Failed to parse: {:#?}", err);
                    }
                };

                $(
                    let $sym = interner.get($text).expect("Get existing symbol");
                )*

                assert_eq!(program, $program);
            }
        };
    }

    make_test!(
        add,
        "$1 = 1 + 2 * 3;",
        [(one, "1"),],
        [Stmt::Assign {
            var: one,
            value: Expr::BinaryOp {
                lhs: Box::new(Expr::Number(1)),
                rhs: Box::new(Expr::BinaryOp {
                    lhs: Box::new(Expr::Number(2)),
                    rhs: Box::new(Expr::Number(3)),
                    op: BinaryOperator::Mul,
                }),
                op: BinaryOperator::Add,
            },
            location: Location::new(1),
        }]
    );

    make_test!(
        print,
        "@ '123' 123;",
        [(text, "123"),],
        [Stmt::Print {
            values: vec![Expr::String(text), Expr::Number(123)],
            newline: true,
            wait: false,
            location: Location::new(1),
        }]
    );

    make_test!(
        variable,
        "
            $1 = 1;
            $2 = 2;
            $3 = $1 + $2;
            ",
        [(one, "1"), (two, "2"), (three, "3"),],
        [
            Stmt::Assign {
                var: one,
                value: Expr::Number(1),
                location: Location::new(2),
            },
            Stmt::Assign {
                var: two,
                value: Expr::Number(2),
                location: Location::new(3),
            },
            Stmt::Assign {
                var: three,
                value: Expr::BinaryOp {
                    lhs: Box::new(Expr::Variable(one)),
                    rhs: Box::new(Expr::Variable(two)),
                    op: BinaryOperator::Add,
                },
                location: Location::new(4),
            },
        ]
    );

    make_test!(
        compare,
        "1 > 2;",
        [],
        [Stmt::Expression {
            expr: Expr::Number(1).binary_op(Expr::Number(2), BinaryOperator::Greater),
            location: Location::new(1),
        }]
    );
}
