use crate::context::Context;
use crate::printer::Printer;

pub trait Builtin {
    fn run<P: Printer>(&mut self, name: &str, ctx: &mut Context<P>);
}

impl<'a, B: Builtin> Builtin for &'a mut B {
    #[inline(always)]
    fn run<P: Printer>(&mut self, name: &str, ctx: &mut Context<P>) {
        (**self).run(name, ctx);
    }
}

pub struct DummyBuiltin;

impl Builtin for DummyBuiltin {
    #[inline(always)]
    fn run<P: Printer>(&mut self, _name: &str, _ctx: &mut Context<P>) {}
}
