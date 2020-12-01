use kes::async_trait;
use kes::builtin::Builtin;
use kes::context::Context;
use kes::parser::parse;
use kes::value::Value;

pub struct StdioBuiltin;

#[async_trait]
impl Builtin for StdioBuiltin {
    #[inline]
    async fn run(&mut self, _name: &str, _ctx: &mut Context<'_>) -> Option<Value> {
        unimplemented!();
    }
    #[inline]
    fn load<'b>(&mut self, _name: &str) -> Value {
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
    let script = parse(include_str!("fib.kes")).unwrap();

    let ctx = Context::new(&script);

    futures::executor::block_on(ctx.run(StdioBuiltin)).unwrap();
}
