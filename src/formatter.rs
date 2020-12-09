use crate::error::ParseError;
use crate::interner::Symbol;
use crate::parser::parse_with_comments;
use crate::{ast::Expr, location::Location};
use crate::{ast::Stmt, interner::Interner};
use std::collections::BTreeMap;
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

impl<'s> From<ParseError> for FormatError {
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
            Expr::String(sym) => write!(f, "'{}'", self.resolve(*sym)),
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
            Expr::Nop(value) => write!(f, "({})", self.display(value)),
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
    comments: &'a BTreeMap<Location, &'a str>,
    last_location: Location,
}

impl<'a, W: Write> CodeFormatter<'a, W> {
    pub fn new(out: W, interner: &'a Interner, comments: &'a BTreeMap<Location, &'a str>) -> Self {
        Self {
            o: IndentWriter::new(out),
            interner,
            comments,
            last_location: Location::new(0),
        }
    }

    pub fn write_program(&mut self, program: &[Stmt]) -> io::Result<()> {
        for stmt in program.iter() {
            self.write_stmt(stmt)?;
        }

        Ok(())
    }

    fn write_stmt_block(&mut self, block: &[Stmt]) -> io::Result<()> {
        self.o.write_all(b"{\n")?;
        self.o.push_block();
        for stmt in block.iter() {
            self.write_stmt(stmt)?;
        }
        self.o.pop_block();
        self.o.write_all(b"}")?;

        Ok(())
    }

    fn write_comment(&mut self, new_location: Location) -> io::Result<()> {
        for (_, comment) in self.comments.range(self.last_location..new_location) {
            writeln!(self.o, "#{}", comment)?;
        }
        self.last_location = new_location;
        Ok(())
    }

    fn write_start_block_comment(
        &mut self,
        blank: bool,
        ident: &str,
        new_location: Location,
    ) -> io::Result<()> {
        let mut comments = self
            .comments
            .range(self.last_location..new_location)
            .peekable();

        let has_comment = comments.peek().is_some();

        if !has_comment {
            if blank {
                write!(self.o, " ")?;
            }
            write!(self.o, "{} ", ident)?;
        } else {
            writeln!(self.o)?;
            for (_, comment) in comments {
                writeln!(self.o, "#{}", comment)?;
            }
            write!(self.o, "{} ", ident)?;
        }

        self.last_location = new_location;
        Ok(())
    }

    pub fn write_stmt(&mut self, stmt: &Stmt) -> io::Result<()> {
        {
            if stmt.is_block() {
                writeln!(self.o)?;
            }
            self.write_comment(stmt.location())?;
        }

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
            Stmt::Exit { .. } => {
                writeln!(self.o, "종료;")?;
            }
            Stmt::If {
                arms,
                other,
                other_location,
            } => {
                let mut first = true;
                for (cond, body, location) in arms.iter() {
                    if first {
                        first = false;
                        self.write_start_block_comment(false, "만약", *location)?;
                    } else {
                        self.write_start_block_comment(true, "혹은", *location)?;
                    }
                    write!(
                        self.o,
                        "{} ",
                        ExprDisplay {
                            expr: cond,
                            interner
                        }
                    )?;
                    self.write_stmt_block(body)?;
                }

                if !other.is_empty() {
                    self.write_start_block_comment(true, "그외", *other_location)?;
                    self.write_stmt_block(other)?;
                }

                self.o.write_all(b"\n\n")?;
            }
            Stmt::While { cond, body, .. } => {
                write!(
                    self.o,
                    "반복 {} ",
                    ExprDisplay {
                        expr: cond,
                        interner
                    }
                )?;
                self.write_stmt_block(body)?;
                self.o.write_all(b"\n\n")?;
            }
            Stmt::Print {
                newline,
                wait,
                values,
                ..
            } => {
                let prefix = if *wait {
                    "@!"
                } else if *newline {
                    "@@"
                } else {
                    "@"
                };

                self.o.write_all(prefix.as_bytes())?;

                for (idx, value) in values.iter().enumerate() {
                    write!(
                        self.o,
                        "{}",
                        ExprDisplay {
                            expr: value,
                            interner
                        }
                    )?;

                    if idx != values.len() - 1 {
                        self.o.write_all(b" ")?;
                    }
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

pub fn format_code(code: &str, out: impl Write) -> Result<(), FormatError> {
    let mut interner = Interner::new();
    let (program, comments) = parse_with_comments(code, &mut interner)?;

    CodeFormatter::new(out, &interner, &comments)
        .write_program(&program)
        .map_err(FormatError::IoError)
}

pub fn format_code_to_string(code: &str) -> Result<String, FormatError> {
    let mut out = Vec::with_capacity(code.len());

    format_code(code, &mut out)?;

    Ok(String::from_utf8(out).unwrap())
}

#[cfg(test)]
mod tests {
    use super::format_code_to_string;
    use crate::builtin::RecordBuiltin;
    use crate::context::Context;
    use crate::program::Program;
    use futures_executor::block_on;

    use pretty_assertions::assert_eq;
    #[test]
    fn simple() {
        assert_eq!(
            format_code_to_string("#12\n$1=2;\n#123\n만약1+2{123;}@!456;").unwrap(),
            "#12\n$1 = 2;\n\n#123\n만약 1 + 2 {\n    123;\n}\n\n@!456;\n"
        );
    }

    #[test]
    fn if_else() {
        assert_eq!(
            format_code_to_string("만약1{123;}혹은2{456;}그외{789;}").unwrap(),
            "\n만약 1 {\n    123;\n} 혹은 2 {\n    456;\n} 그외 {\n    789;\n}\n\n",
        )
    }

    #[test]
    fn if_else_comment() {
        assert_eq!(
            format_code_to_string("만약1{123;\n#comment\n#comment2\n}혹은2{456;}그외{789;}")
                .unwrap(),
            "
만약 1 {
    123;
}
#comment
#comment2
혹은 2 {
    456;
} 그외 {
    789;
}

",
        )
    }

    #[test]
    fn end_comment() {
        assert_eq!(
            format_code_to_string("$1=2;#12\n$2=3;").unwrap(),
            "$1 = 2;\n#12\n$2 = 3;\n"
        );
    }

    #[test]
    fn paren_test() {
        assert_eq!(format_code_to_string("1*(2+3);").unwrap(), "1 * (2 + 3);\n");
    }

    #[test]
    fn work() {
        let code = "$1=2;만약1+2{@@123;}@!456;";
        let formatted_code = format_code_to_string(code).unwrap();

        let mut ori_builtin = RecordBuiltin::new();
        let mut for_builtin = RecordBuiltin::new();

        block_on(Context::new(&Program::from_source(code).unwrap()).run(&mut ori_builtin)).unwrap();
        block_on(
            Context::new(&Program::from_source(&formatted_code).unwrap()).run(&mut for_builtin),
        )
        .unwrap();

        assert_eq!(ori_builtin.text(), for_builtin.text());
    }
}
