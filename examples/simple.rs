use kes::builtin::DummyBuiltin;
use kes::bumpalo::Bump;
use kes::context::Context;
use kes::parser::parse;
use kes::printer::DummyPrinter;

fn main() {
    let bump = Bump::with_capacity(8196);
    let script = parse(&bump, "1 2 + 3 - 0 <> '1' [-]");

    for _ in 0..1000000 {
        let ctx = Context::new(&bump, &script, DummyPrinter);
        ctx.run(DummyBuiltin);
    }
}
