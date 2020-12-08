use crate::ast::Expr;
use crate::error::ParseError;
use crate::interner::Symbol;
use crate::parser::parse;
use crate::{ast::Stmt, interner::Interner};
use std::fmt;
use std::io::{self, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("파싱에러: {0:?}")]
    ParseError(ParseError),
    #[error("IO 에러: {0}")]
    IoError(#[from] io::Error),
}

impl From<ParseError> for FormatError {
    fn from(err: ParseError) -> Self {
        FormatError::ParseError(err)
    }
}

struct IndentWriter<W: Write> {
    out: W,
    indent_writed: bool,
    block: usize,
}

impl<W: Write> IndentWriter<W> {
    pub fn new(out: W) -> Self {
        Self {
            out,
            indent_writed: false,
            block: 0,
        }
    }

    pub fn push_block(&mut self) {
        self.block += 1;
    }

    pub fn pop_block(&mut self) {
        self.block -= 1;
    }
}

impl<W: Write> Write for IndentWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        const INDENT: &str = "    ";

        if !self.indent_writed {
            for _ in 0..self.block {
                self.out.write(INDENT.as_bytes())?;
            }

            self.indent_writed = true;
        }

        if memchr::memchr(b'\n', buf).is_some() {
            self.indent_writed = false;
        }

        self.out.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

struct ExprDisplay<'a> {
    expr: &'a Expr,
    interner: &'a Interner,
}

impl<'a> ExprDisplay<'a> {
    fn display(&self, expr: &'a Expr) -> Self {
        Self {
            expr,
            interner: self.interner,
        }
    }

    fn resolve(&self, sym: Symbol) -> &str {
        self.interner.resolve(sym).unwrap()
    }
}

impl<'a> fmt::Display for ExprDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.expr {
            Expr::Number(num) => write!(f, "{}", num),
            Expr::String(sym) => write!(f, "{}", self.resolve(*sym)),
            Expr::Variable(sym) => write!(f, "${}", self.resolve(*sym)),
            Expr::BuiltinFunc { name, args } => {
                write!(f, "{}(", self.resolve(*name))?;

                for (idx, arg) in args.iter().enumerate() {
                    write!(f, "{}", self.display(arg))?;
                    if idx != args.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                write!(f, ")")
            }
            Expr::BinaryOp { lhs, rhs, op } => {
                write!(
                    f,
                    "{} {} {}",
                    self.display(lhs),
                    op.name(),
                    self.display(rhs)
                )
            }
            Expr::UnaryOp { value, op } => {
                write!(f, "{}{}", op.name(), self.display(value))
            }
            Expr::TernaryOp { lhs, mhs, rhs, op } => {
                write!(
                    f,
                    "{} {} {} {} {}",
                    self.display(lhs),
                    op.first_name(),
                    self.display(mhs),
                    op.second_name(),
                    self.display(rhs)
                )
            }
        }
    }
}

struct CodeFormatter<'a, W: Write> {
    o: IndentWriter<W>,
    interner: &'a Interner,
}

impl<'a, W: Write> CodeFormatter<'a, W> {
    pub fn new(out: W, interner: &'a Interner) -> Self {
        Self {
            o: IndentWriter::new(out),
            interner,
        }
    }

    pub fn write_program(&mut self, program: &[Stmt]) -> io::Result<()> {
        for stmt in program.iter() {
            self.write_stmt(stmt)?;
        }

        Ok(())
    }

    fn write_stmt_block(&mut self, block: &[Stmt]) -> io::Result<()> {
        self.o.push_block();
        for stmt in block.iter() {
            self.write_stmt(stmt)?;
        }
        self.o.pop_block();

        Ok(())
    }

    pub fn write_stmt(&mut self, stmt: &Stmt) -> io::Result<()> {
        let interner = self.interner;

        macro_rules! res {
            ($sym:expr) => {
                interner.resolve($sym).unwrap()
            };
        }

        match stmt {
            Stmt::Assign { var, value, .. } => {
                writeln!(
                    self.o,
                    "${} = {};",
                    res!(*var),
                    ExprDisplay {
                        expr: &value,
                        interner
                    }
                )?;
            }
            Stmt::Exit => writeln!(self.o, "종료;")?,
            Stmt::If { arms, other, .. } => {
                let mut first = true;
                for (cond, body) in arms.iter() {
                    if first {
                        write!(self.o, "만약")?;
                        first = false;
                    } else {
                        write!(self.o, "혹은")?;
                    }
                    writeln!(
                        self.o,
                        " {} {{",
                        ExprDisplay {
                            expr: cond,
                            interner
                        }
                    )?;
                    self.write_stmt_block(body)?;
                    write!(self.o, "}}")?;
                }

                if !other.is_empty() {
                    writeln!(self.o, "그외 {{")?;
                    self.write_stmt_block(other)?;
                    write!(self.o, "}}")?;
                }

                write!(self.o, "\n\n")?;
            }
            Stmt::While { cond, body, .. } => {
                writeln!(
                    self.o,
                    "반복 {} {{",
                    ExprDisplay {
                        expr: cond,
                        interner
                    }
                )?;
                self.write_stmt_block(body)?;
                write!(self.o, "}}\n\n")?;
            }
            Stmt::Print {
                newline,
                wait,
                values,
                ..
            } => {
                let prefix = if *wait {
                    "@! "
                } else if *newline {
                    "@ "
                } else {
                    "@@ "
                };

                self.o.write_all(prefix.as_bytes())?;

                for value in values.iter() {
                    write!(
                        self.o,
                        "{} ",
                        ExprDisplay {
                            expr: value,
                            interner
                        }
                    )?;
                }

                writeln!(self.o, ";")?;
            }
            Stmt::Expression { expr, .. } => {
                writeln!(self.o, "{};", ExprDisplay { expr, interner })?;
            }
        }

        Ok(())
    }
}

pub fn format_program(
    program: &[Stmt],
    interner: &Interner,
    out: impl Write,
) -> Result<(), io::Error> {
    CodeFormatter::new(out, interner).write_program(program)
}

pub fn format_code(code: &str, out: impl Write) -> Result<(), FormatError> {
    let mut interner = Interner::new();
    let program = parse(code, &mut interner)?;

    format_program(&program, &interner, out).map_err(FormatError::IoError)
}

pub fn format_code_to_string(code: &str) -> Result<String, FormatError> {
    let mut out = Vec::with_capacity(code.len());

    format_code(code, &mut out)?;

    Ok(String::from_utf8(out).unwrap())
}

#[test]
fn simple() {
    use pretty_assertions::assert_eq;
    assert_eq!(
        format_code_to_string("$1=2;만약1+2{123;}@!456;").unwrap(),
        "$1 = 2;\n만약 1 + 2 {\n    123;\n}\n\n@! 456 ;\n"
    );
}
