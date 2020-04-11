use kes::async_trait;
use kes::builtin::DummyBuiltin;
use kes::bumpalo::Bump;
use kes::console::KesConsole;
use kes::context::Context;
use kes::parser::parse;
use kes::value::Value;

pub struct StdioPrinter;

#[async_trait(?Send)]
impl KesConsole for StdioPrinter {
    #[inline]
    fn print(&mut self, v: Value) {
        print!("{}", v);
    }
    #[inline]
    fn new_line(&mut self) {
        println!();
    }
    #[inline]
    async fn wait(&mut self) {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
    }
}

fn main() {
    let bump = Bump::with_capacity(8196);

    let script = parse(&bump, include_str!("fib.kes")).unwrap();

    let ctx = Context::new(&bump, &script);

    futures::executor::block_on(ctx.run(DummyBuiltin, StdioPrinter));
}
