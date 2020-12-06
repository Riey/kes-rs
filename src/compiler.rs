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
            Stmt::If { .. } | Stmt::While { .. } => todo!(),
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
    fn if_simple() {
        assert_eq!(
            compile_source("만약 1 + 2 { }").unwrap(),
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::Add),
                Instruction::GotoIfNot(1),
                Instruction::Goto(1),
            ]
        );
    }
}
