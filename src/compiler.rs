use crate::location::Location;
use crate::{ast::Expr, ast::Stmt};
use crate::{
    error::ParseError,
    instruction::{Instruction, InstructionWithDebug},
};
use arrayvec::ArrayVec;

pub struct Compiler<'s> {
    out: Vec<InstructionWithDebug<'s>>,
    location: Location,
}

impl<'s> Compiler<'s> {
    pub fn new() -> Self {
        Self {
            out: Vec::new(),
            location: Location::default(),
        }
    }

    fn push(&mut self, inst: Instruction<'s>) {
        self.out.push(InstructionWithDebug {
            inst,
            location: self.location,
        });
    }

    fn next_pos(&self) -> u32 {
        self.out.len() as u32
    }

    fn mark_pos(&mut self) -> u32 {
        let next = self.next_pos();
        self.push(Instruction::Nop);
        next
    }

    fn compile_stmt(&mut self, stmt: &Stmt<'s>) {
        match stmt {
            Stmt::Print {
                values,
                newline,
                wait,
                location,
            } => {
                self.location = *location;
                for value in values {
                    self.push_expr(value);
                }
                self.push(Instruction::Print {
                    wait: *wait,
                    newline: *newline,
                });
            }
            Stmt::Assign {
                var,
                value,
                location,
            } => {
                self.location = *location;
                self.push_expr(value);
                self.push(Instruction::StoreVar(*var));
            }
            Stmt::Expression { expr, location } => {
                self.location = *location;
                self.push_expr(expr);
                self.push(Instruction::Pop);
            }
            Stmt::If {
                arms,
                other,
                location,
            } => {
                self.location = *location;
                let mut mark = 0;
                let mut else_mark = ArrayVec::<[_; 20]>::new();

                for (idx, (cond, body)) in arms.iter().enumerate() {
                    let first = idx == 0;

                    if !first {
                        self.out[mark as usize].inst =
                            Instruction::GotoIfNot(self.next_pos() as u32);
                    }

                    self.push_expr(cond);

                    mark = self.mark_pos();

                    self.compile_body(body);
                    else_mark.push(self.mark_pos());
                }

                if !arms.is_empty() {
                    self.out[mark as usize].inst = Instruction::GotoIfNot(self.next_pos() as u32);
                }

                self.compile_body(other);

                for mark in else_mark {
                    self.out[mark as usize].inst = Instruction::Goto(self.next_pos() as u32);
                }
            }
            Stmt::While {
                cond,
                body,
                location,
            } => {
                self.location = *location;
                let first = self.next_pos();
                self.push_expr(cond);
                let end = self.mark_pos();

                self.compile_body(body);
                self.push(Instruction::Goto(first as u32));
                self.out[end as usize].inst = Instruction::GotoIfNot(self.next_pos() as u32);
            }
        }
    }

    fn push_expr(&mut self, expr: &Expr<'s>) {
        match expr {
            Expr::Number(num) => self.push(Instruction::LoadInt(*num)),
            Expr::String(str) => self.push(Instruction::LoadStr(*str)),
            Expr::Variable(var) => self.push(Instruction::LoadVar(*var)),
            Expr::UnaryOp { value, op } => {
                self.push_expr(value);
                self.push(Instruction::UnaryOperator(*op));
            }
            Expr::BinaryOp { lhs, rhs, op } => {
                self.push_expr(lhs);
                self.push_expr(rhs);
                self.push(Instruction::BinaryOperator(*op));
            }
            Expr::TernaryOp { lhs, mhs, rhs, op } => {
                self.push_expr(lhs);
                self.push_expr(mhs);
                self.push_expr(rhs);
                self.push(Instruction::TernaryOperator(*op));
            },
        }
    }

    fn compile_body(&mut self, body: &[Stmt<'s>]) {
        for stmt in body.iter() {
            self.compile_stmt(stmt);
        }
    }

    pub fn compile(mut self, program: &[Stmt<'s>]) -> Vec<InstructionWithDebug<'s>> {
        self.compile_body(program);
        self.out
    }
}

pub fn compile<'s>(program: &[Stmt<'s>]) -> Vec<InstructionWithDebug<'s>> {
    Compiler::new().compile(program)
}

pub fn compile_source<'s>(source: &'s str) -> Result<Vec<InstructionWithDebug<'s>>, ParseError> {
    Ok(compile(&crate::parser::parse(source)?))
}

#[cfg(test)]
mod tests {
    use super::compile_source;
    use crate::operator::TernaryOperator;
    use crate::{instruction::Instruction, operator::BinaryOperator};
    use pretty_assertions::assert_eq;

    fn test_impl(source: &str, insts: &[Instruction]) {
        let compiled = compile_source(source)
            .unwrap()
            .into_iter()
            .map(|i| i.inst)
            .collect::<Vec<_>>();

        assert_eq!(compiled, insts);
    }

    #[test]
    fn simple() {
        test_impl(
            "1 + 2;",
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::Add),
                Instruction::Pop,
            ],
        );
    }

    #[test]
    fn print() {
        test_impl(
            "@ 123 '123';",
            &[
                Instruction::LoadInt(123),
                Instruction::LoadStr("123"),
                Instruction::Print {
                    newline: true,
                    wait: false,
                },
            ],
        )
    }

    #[test]
    fn if_simple() {
        test_impl(
            "만약 1 + 2 { 0; } 혹은 1 { 1; } 그외 { 2; }",
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::Add),
                Instruction::GotoIfNot(7),
                Instruction::LoadInt(0),
                Instruction::Pop,
                Instruction::Goto(14),
                Instruction::LoadInt(1),
                Instruction::GotoIfNot(12),
                Instruction::LoadInt(1),
                Instruction::Pop,
                Instruction::Goto(14),
                Instruction::LoadInt(2),
                Instruction::Pop,
            ],
        );
    }

    #[test]
    fn while_simple() {
        test_impl(
            "반복 1 + 2 { 2; } 3;",
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::Add),
                Instruction::GotoIfNot(7),
                Instruction::LoadInt(2),
                Instruction::Pop,
                Instruction::Goto(0),
                Instruction::LoadInt(3),
                Instruction::Pop,
            ],
        );
    }

    #[test]
    fn conditional() {
        test_impl(
            "1 ? 2 : 3;",
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::LoadInt(3),
                Instruction::TernaryOperator(TernaryOperator::Conditional),
                Instruction::Pop,
            ],
        );
    }
}
