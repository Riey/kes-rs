use kes::Interpreter;
use kes::printer::DummyPrinter;
use bumpalo::Bump;

fn main() {
    let bump = Bump::with_capacity(8196);
    let mut interpreter = Interpreter::new(&bump);
    interpreter.load_script("foo", "1 2 + 3 - 0 <> '1' [-]");

    for _ in 0..1000000 {
        interpreter.run_script("foo", &mut DummyPrinter);
    }
}
