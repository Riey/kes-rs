pub mod builtin;
pub mod context;
pub mod instruction;
pub mod lexer;
pub mod operator;
pub mod parser;
pub mod printer;
pub mod token;

use crate::builtin::Builtin;
use crate::instruction::Instruction;
use crate::{context::Context, printer::Printer};
use ahash::AHashMap;
use bumpalo::collections::{String, Vec};
pub use bumpalo::Bump;

pub struct Interpreter<'b> {
    bump: &'b Bump,
    exe_bump: Bump,
    scripts: AHashMap<&'b str, Vec<'b, Instruction<'b>>>,
    print_buffer: String<'b>,
}

impl<'b> Interpreter<'b> {
    pub fn new(bump: &'b Bump) -> Self {
        Self {
            bump,
            exe_bump: Bump::with_capacity(1024 * 1024),
            scripts: AHashMap::new(),
            print_buffer: String::with_capacity_in(1024 * 10, bump),
        }
    }

    #[inline]
    pub fn bump(&self) -> &'b Bump {
        self.bump
    }

    pub fn load_script(&mut self, name: &'b str, source: &str) {
        self.scripts.insert(
            name,
            crate::parser::parse(self.bump, crate::lexer::lex(source)),
        );
    }

    pub fn run_script<B: Builtin, P: Printer>(
        &mut self,
        builtin: B,
        name: &str,
        printer: P,
    ) -> bool {
        if let Some(script) = self.scripts.get(name) {
            let ctx = Context::new(&self.exe_bump, script, printer, &mut self.print_buffer);
            ctx.run(builtin);
            self.print_buffer.clear();
            self.exe_bump.reset();
            true
        } else {
            false
        }
    }

    pub fn eval<B: Builtin, P: Printer>(&mut self, builtin: &mut B, source: &str, printer: P) {
        let bump = &self.exe_bump;
        let script = crate::parser::parse(&self.bump, crate::lexer::lex(source));
        let ctx = Context::new(bump, &script, printer, &mut self.print_buffer);
        ctx.run(builtin);
        self.print_buffer.clear();
        self.exe_bump.reset();
    }
}

#[test]
fn interpreter_run_test() {
    use crate::builtin::DummyBuiltin;
    use crate::printer::RecordPrinter;
    let bump = Bump::with_capacity(8196);
    let mut interpreter = Interpreter::new(&bump);
    interpreter.load_script(
        "foo",
        "
1 2 + 3 == 2 3 [?]
'1 + 2 = ' 1 2 + #
5 2 % 7 + [$4]
$4
",
    );
    let mut printer = RecordPrinter::new();

    let ret = interpreter.run_script(DummyBuiltin, "foo", &mut printer);

    assert!(ret);
    assert_eq!(printer.text(), "21 + 2 = 3\n#8");
}
