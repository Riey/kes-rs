pub mod context;
pub mod instruction;
pub mod lexer;
pub mod operator;
pub mod parser;
pub mod printer;
pub mod token;

use crate::context::Context;
use crate::instruction::Instruction;
use crate::printer::Printer;
use bumpalo::collections::{String, Vec};
pub use bumpalo::Bump;
use std::collections::HashMap;

pub struct Interpreter<'b, P: Printer> {
    bump: &'b Bump,
    exe_bump: Bump,
    builtin: HashMap<&'b str, fn(&mut Context<'b, '_, P>)>,
    scripts: HashMap<&'b str, Vec<'b, Instruction<'b>>>,
    print_buffer: String<'b>,
}

impl<'b, P: Printer> Interpreter<'b, P> {
    pub fn new(bump: &'b Bump) -> Self {
        Self {
            bump,
            exe_bump: Bump::with_capacity(1024 * 1024),
            builtin: HashMap::new(),
            scripts: HashMap::new(),
            print_buffer: String::with_capacity_in(1024 * 10, bump),
        }
    }

    #[inline]
    pub fn bump(&self) -> &'b Bump {
        self.bump
    }


    pub fn insert_builtin(&mut self, name: &'b str, func: fn(&mut Context<'b, '_, P>)) {
        self.builtin.insert(name, func);
    }

    pub fn load_script(&mut self, name: &'b str, source: &str) {
        self.scripts.insert(
            name,
            crate::parser::parse(self.bump, crate::lexer::lex(source)),
        );
    }

    pub fn run_script(&mut self, name: &str, printer: &mut P) -> bool {
        if let Some(script) = self.scripts.get(name) {
            let ctx = crate::context::Context::new(
                &self.exe_bump,
                &self.builtin,
                script,
                printer,
                &mut self.print_buffer,
            );
            ctx.run();
            self.print_buffer.clear();
            self.exe_bump.reset();
            true
        } else {
            false
        }
    }
}

#[test]
fn interpreter_run_test() {
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

    let ret = interpreter.run_script("foo", &mut printer);

    assert!(ret);
    assert_eq!(printer.text(), "21 + 2 = 3\n#8");
}
