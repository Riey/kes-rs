use bumpalo::Bump;
use kes::printer::DummyPrinter;
use kes::Interpreter;

fn main() {
    let bump = Bump::with_capacity(8196);
    let mut interpreter = Interpreter::new(&bump);
    interpreter.load_script("foo", "1 2 + 3 - 0 <> '1' [-]");
    let builtin = Default::default();

    for _ in 0..1000000 {
        interpreter.run_script(&builtin, "foo", DummyPrinter);
    }
}
