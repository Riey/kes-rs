use kes::builtin::DummyBuiltin;
use kes::bumpalo::Bump;
use kes::context::Context;
use kes::parser::parse;
use kes::printer::StdioPrinter;

fn main() {
    let bump = Bump::with_capacity(8196);

    let script = parse(&bump, include_str!("fib.kes"));

    let ctx = Context::new(&bump, &script, StdioPrinter);

    ctx.run(DummyBuiltin);
}
