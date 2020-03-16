use kes::builtin::Builtin;
use kes::bumpalo::Bump;
use kes::context::Context;
use kes::parser::parse;
use kes::value::Value;

pub struct StdioBuiltin;

impl Builtin for StdioBuiltin {
    #[inline(always)]
    fn run(&mut self, _name: &str, _ctx: &mut Context) {
        unimplemented!();
    }
    #[inline(always)]
    fn load(&mut self, _name: &str, _ctx: &mut Context) {
        unimplemented!();
    }
    #[inline(always)]
    fn print(&mut self, v: Value) {
        print!("{}", v);
    }
    #[inline(always)]
    fn new_line(&mut self) {
        println!();
    }
    #[inline(always)]
    fn wait(&mut self) {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
    }
}

fn main() {
    let bump = Bump::with_capacity(8196);

    let script = parse(&bump, include_str!("fib.kes")).unwrap();

    let ctx = Context::new(&bump, &script);

    ctx.run(StdioBuiltin);
}
