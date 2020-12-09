use crate::instruction::{Instruction, InstructionWithDebug};
use crate::location::Location;
use crate::{ast::Expr, ast::Stmt};
use arrayvec::ArrayVec;

pub struct Compiler {
    out: Vec<InstructionWithDebug>,
    location: Location,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            out: Vec::new(),
            location: Location::default(),
        }
    }

    fn push(&mut self, inst: Instruction) {
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

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Exit { location } => {
                self.location = *location;
                self.push(Instruction::Exit);
            }
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

    fn push_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(num) => self.push(Instruction::LoadInt(*num)),
            Expr::String(str) => self.push(Instruction::LoadStr(*str)),
            Expr::Variable(var) => self.push(Instruction::LoadVar(*var)),
            Expr::BuiltinFunc { name, args } => {
                for arg in args.iter() {
                    self.push_expr(arg);
                }
                self.push(Instruction::CallBuiltin(*name));
            }
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
            }
        }
    }

    fn compile_body(&mut self, body: &[Stmt]) {
        for stmt in body.iter() {
            self.compile_stmt(stmt);
        }
    }

    pub fn compile(mut self, program: &[Stmt]) -> Vec<InstructionWithDebug> {
        self.compile_body(program);
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::Compiler;
    use crate::interner::Interner;
    use crate::operator::TernaryOperator;
    use crate::parser::parse;
    use crate::{instruction::Instruction, operator::BinaryOperator};
    use pretty_assertions::assert_eq;

    fn test_impl(source: &str, interner: &mut Interner, insts: &[Instruction]) {
        let ast = parse(source, interner).unwrap();
        let compiled = Compiler::new()
            .compile(&ast)
            .into_iter()
            .map(|i| i.inst)
            .collect::<Vec<_>>();

        assert_eq!(compiled, insts);
    }

    #[test]
    fn simple() {
        let mut i = Interner::new();
        test_impl(
            "1 + 2;",
            &mut i,
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::Add),
                Instruction::Pop,
            ],
        );
    }

    #[test]
    fn and_or() {
        let mut i = Interner::new();
        test_impl(
            "1 | 3 & 2;",
                &mut i,
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(3),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::And),
                Instruction::BinaryOperator(BinaryOperator::Or),
                Instruction::Pop,
            ]
        );
    }

    #[test]
    fn print() {
        let mut i = Interner::new();
        let foo = i.get_or_intern_static("123");
        test_impl(
            "@@123 '123';",
            &mut i,
            &[
                Instruction::LoadInt(123),
                Instruction::LoadStr(foo),
                Instruction::Print {
                    newline: true,
                    wait: false,
                },
            ],
        )
    }

    #[test]
    fn comment() {
        let mut i = Interner::new();
        test_impl(
            "  #----\n  123;",
            &mut i,
            &[Instruction::LoadInt(123), Instruction::Pop],
        );
    }

    #[test]
    fn if_simple() {
        let mut i = Interner::new();
        test_impl(
            "만약 1 + 2 { 0; } 혹은 1 { 1; } 그외 { 2; }",
            &mut i,
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
        let mut i = Interner::new();
        test_impl(
            "반복 1 + 2 { 2; } 3;",
            &mut i,
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
        let mut i = Interner::new();
        test_impl(
            "1 ? 2 : 3;",
            &mut i,
            &[
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::LoadInt(3),
                Instruction::TernaryOperator(TernaryOperator::Conditional),
                Instruction::Pop,
            ],
        );
    }

    #[test]
    fn builtin() {
        let mut i = Interner::new();
        let v = i.get_or_intern_static("변수");
        let f = i.get_or_intern_static("함수");
        test_impl(
            "변수(); 함수(1 + 2);",
            &mut i,
            &[
                Instruction::CallBuiltin(v),
                Instruction::Pop,
                Instruction::LoadInt(1),
                Instruction::LoadInt(2),
                Instruction::BinaryOperator(BinaryOperator::Add),
                Instruction::CallBuiltin(f),
                Instruction::Pop,
            ],
        );
    }

    #[test]
    fn exit() {
        let mut i = Interner::new();
        test_impl("종료;", &mut i, &[Instruction::Exit]);
    }
}
