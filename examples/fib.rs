use kes::builtin::Builtin;
use kes::bumpalo::Bump;
use kes::context::Context;
use kes::error::RuntimeResult;
use kes::parser::parse;
use kes::value::Value;

pub struct StdioBuiltin;

impl Builtin for StdioBuiltin {
    #[inline]
    fn run<'c>(
        &mut self,
        _name: &'c str,
        _ctx: &mut Context<'c>,
    ) -> RuntimeResult<Option<Value<'c>>> {
        unimplemented!()
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

    let script = parse(&bump, include_str!("fib.kes")).unwrap();

    let mut ctx = Context::new(&bump, &script);

    ctx.run(StdioBuiltin).unwrap();
}
