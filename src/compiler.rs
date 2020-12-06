use crate::{ast::Expr, ast::Stmt};
use crate::{error::ParseError, instruction::Instruction};

pub struct Compiler<'s> {
    out: Vec<Instruction<'s>>,
}

impl<'s> Compiler<'s> {
    pub fn new() -> Self {
        Self { out: Vec::new() }
    }

    fn push(&mut self, inst: Instruction<'s>) {
        self.out.push(inst);
    }

    fn next_pos(&self) -> usize {
        self.out.len()
    }

    fn mark_pos(&mut self) -> usize {
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
            } => {
                for value in values {
                    self.push_expr(value);
                }
                self.push(Instruction::Print {
                    wait: *wait,
                    newline: *newline,
                });
            }
            Stmt::Assign { var, value } => {
                self.push_expr(value);
                self.push(Instruction::StoreVar(*var));
            }
            Stmt::Expression(expr) => {
                self.push_expr(expr);
                self.push(Instruction::Pop);
            }
            Stmt::If { arms, other } => {
                let mut mark = 0;
                let mut else_mark = Vec::with_capacity(arms.len());

                for (idx, (cond, body)) in arms.iter().enumerate() {
                    let first = idx == 0;

                    if !first {
                        self.out[mark] = Instruction::GotoIfNot(self.next_pos() as u32);
                    }

                    self.push_expr(cond);

                    mark = self.mark_pos();

                    self.compile_body(body);
                    else_mark.push(self.mark_pos());
                }

                if !arms.is_empty() {
                    self.out[mark] = Instruction::GotoIfNot(self.next_pos() as u32);
                }

                self.compile_body(other);

                for mark in else_mark {
                    self.out[mark] = Instruction::Goto(self.next_pos() as u32);
                }
            }
            Stmt::While { cond, body } => {
                let first = self.next_pos();
                self.push_expr(cond);
                let end = self.mark_pos();

                self.compile_body(body);
                self.push(Instruction::Goto(first as u32));
                self.out[end] = Instruction::GotoIfNot(self.next_pos() as u32);
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
        }
    }

    fn compile_body(&mut self, body: &[Stmt<'s>]) {
        for stmt in body.iter() {
            self.compile_stmt(stmt);
        }
    }

    pub fn compile(mut self, program: &[Stmt<'s>]) -> Vec<Instruction<'s>> {
        self.compile_body(program);
        self.out
    }
}

pub fn compile<'s>(program: &[Stmt<'s>]) -> Vec<Instruction<'s>> {
    Compiler::new().compile(program)
}

pub fn compile_source<'s>(source: &'s str) -> Result<Vec<Instruction<'s>>, ParseError> {
    Ok(compile(&crate::parser::parse(source)?))
}

#[cfg(test)]
mod tests {
    use super::compile_source;
    use crate::{instruction::Instruction, operator::BinaryOperator};
    use pretty_assertions::assert_eq;

    #[test]
    fn simple() {
        assert_eq!(
            compile_source("1 + 2;").unwrap(),
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::Add),
                Instruction::Pop,
            ]
        );
    }

    #[test]
    fn print() {
        assert_eq!(
            compile_source("@ 123 '123';").unwrap(),
            &[
                Instruction::LoadInt(123),
                Instruction::LoadStr("123"),
                Instruction::Print {
                    newline: true,
                    wait: false
                },
            ]
        )
    }

    #[test]
    fn if_simple() {
        assert_eq!(
            compile_source("만약 1 + 2 { 0; } 혹은 1 { 1; } 그외 { 2; }").unwrap(),
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
            ]
        );
    }

    #[test]
    fn while_simple() {
        assert_eq!(
            compile_source("반복 1 + 2 { 2; } 3;").unwrap(),
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
            ]
        )
    }
}
