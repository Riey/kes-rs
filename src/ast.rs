use crate::token::{OperatorToken, StrLitPostFix, Token};
use std::iter::Peekable;

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
pub enum Expr<'s> {
    IntLit(u32),
    StrLit(&'s str),
    Binary(Box<Expr<'s>>, BinaryOp, Box<Expr<'s>>),
    Unary(UnaryOp, Box<Expr<'s>>),
    Conditional(Box<Expr<'s>>, Box<Expr<'s>>, Box<Expr<'s>>),
    Var(&'s str),
    Builtin(&'s str, Vec<Expr<'s>>),
}

#[derive(Clone, Debug)]
pub struct PrintArg<'s> {
    first: &'s str,
    left: Vec<(Expr<'s>, &'s str)>,
    new_line: bool,
    wait: bool,
}

#[derive(Clone, Debug)]
pub enum Stmt<'s> {
    Assign(&'s str, Expr<'s>),
    Print(PrintArg<'s>),
    Goto(usize),
    GotoIf(usize, Expr<'s>),
}

struct Parser<'s, I: Iterator<Item = Token<'s>>> {
    tokens: Peekable<I>,
    stmts: Vec<Stmt<'s>>,
}

impl<'s, I: Iterator<Item = Token<'s>>> Parser<'s, I> {
    fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            stmts: Vec::with_capacity(10000),
        }
    }

    fn next_expr(&mut self) -> Expr<'s> {
        match self.tokens.next().unwrap() {
            Token::OpenParen => {
                let expr = self.next_expr();
                assert_eq!(
                    self.tokens.next(),
                    Some(Token::CloseParen),
                    "Param is not paired"
                );
                expr
            }
            Token::IntLit(num) => Expr::IntLit(num),
            Token::StrLit(text, _) => Expr::StrLit(text),
            Token::Builtin(ident) => {
                if self.tokens.peek() == Some(&Token::OpenParen) {
                    self.tokens.next();

                    let mut buf = Vec::with_capacity(10);

                    loop {
                        buf.push(self.next_expr());

                        match self.tokens.next() {
                            Some(Token::CloseParen) => break,
                            Some(Token::Comma) if self.tokens.peek() == Some(&Token::CloseParen) => {
                                self.tokens.next();
                                break;
                            }
                            Some(Token::Comma) => {
                                continue;
                            }
                            token => panic!("Unexpected token: {:?}", token),
                        }
                    }

                    Expr::Builtin(ident, buf)
                } else {
                    Expr::Builtin(ident, Vec::new())
                }
            }
            //Token::Variable(ident) =>
            token => panic!("Unexpected token {:?}", token),
        }
    }

    fn parse(mut self) -> Vec<Stmt<'s>> {
        while let Some(token) = self.tokens.next() {
            match token {
                Token::OpenParen => todo!(),
                token => panic!("Unexpected token {:?}", token),
            }
        }

        self.stmts
    }
}

pub fn parse<'s>(tokens: impl Iterator<Item = Token<'s>>) -> Vec<Stmt<'s>> {
    Parser::new(tokens).parse()
}
