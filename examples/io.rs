use kes::builtin::Builtin;
use kes::bumpalo::Bump;
use kes::context::Context;
use kes::error::{RuntimeError, RuntimeResult};
use kes::parser::parse;
use kes::value::Value;

pub struct StdioBuiltin;

impl Builtin for StdioBuiltin {
    #[inline]
    fn run<'c>(
        &mut self,
        name: &'c str,
        ctx: &mut Context<'c>,
    ) -> RuntimeResult<Option<Value<'c>>> {
        match name {
            "읽기" => Err(ctx.set_interrupt()),
            _ => Err(ctx.make_err("알수없는 함수명")),
        }
    }
    #[inline]
    fn load<'c>(&mut self, _name: &'c str, _ctx: &mut Context<'c>) -> RuntimeResult<Value<'c>> {
        unimplemented!();
    }
    #[inline]
    fn print(&mut self, v: Value) {
        print!("{}", v);
    }
    #[inline]
    fn new_line(&mut self) {
        println!();
    }
}

fn main() {
    let bump = Bump::with_capacity(8196);

    let script = parse(&bump, include_str!("io.kes")).unwrap();

    let mut ctx = Context::new(&bump, &script);
    let mut builtin = StdioBuiltin;

    loop {
        match ctx.run(&mut builtin) {
            Ok(_) => break,
            Err(RuntimeError::Interrupted(state)) => {
                eprintln!("Interrupted: {}", state);
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).unwrap();
                ctx.resume(Some(Value::Str(ctx.bump.alloc_str(&buf))));
            }
            Err(other) => {
                eprintln!("Runtime error: {}", other);
                return;
            }
        }
    }
}
