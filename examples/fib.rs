use kes::async_trait;
use kes::builtin::Builtin;
use kes::bumpalo::Bump;
use kes::context::Context;
use kes::parser::parse;
use kes::value::Value;

pub struct StdioBuiltin;

#[async_trait(?Send)]
impl Builtin for StdioBuiltin {
    #[inline]
    async fn run<'c>(&mut self, _name: &'_ str, _ctx: &'_ mut Context<'c>) -> Option<Value<'c>> {
        unimplemented!();
    }
    #[inline]
    fn load<'b>(&mut self, _name: &str, _b: &'b Bump) -> Value<'b> {
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

    futures::executor::block_on(ctx.run(StdioBuiltin));
}
