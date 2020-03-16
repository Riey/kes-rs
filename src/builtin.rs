use crate::context::Context;
use crate::value::Value;

pub trait Builtin {
    fn run(&mut self, name: &str, ctx: &mut Context);
    fn load(&mut self, name: &str, ctx: &mut Context);
    fn print(&mut self, v: Value);
    fn new_line(&mut self);
    fn wait(&mut self);
}

impl<'a, B: Builtin> Builtin for &'a mut B {
    #[inline]
    fn run(&mut self, name: &str, ctx: &mut Context) {
        (**self).run(name, ctx);
    }
    #[inline]
    fn load(&mut self, name: &str, ctx: &mut Context) {
        (**self).run(name, ctx);
    }
    #[inline]
    fn print(&mut self, v: Value) {
        (**self).print(v);
    }
    #[inline]
    fn new_line(&mut self) {
        (**self).new_line();
    }
    #[inline]
    fn wait(&mut self) {
        (**self).wait();
    }
}

pub struct DummyBuiltin;

impl Builtin for DummyBuiltin {
    #[inline]
    fn run(&mut self, _name: &str, _ctx: &mut Context) {}
    #[inline]
    fn load(&mut self, _name: &str, _ctx: &mut Context) {}
    #[inline]
    fn print(&mut self, _v: Value) {}
    #[inline]
    fn new_line(&mut self) {}
    #[inline]
    fn wait(&mut self) {}
}

pub struct RecordBuiltin(String);

impl RecordBuiltin {
    #[inline]
    pub fn new() -> Self {
        Self(String::with_capacity(8196))
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.0
    }
}

impl Builtin for RecordBuiltin {
    #[inline]
    fn run(&mut self, name: &str, _ctx: &mut Context) {
        self.0.push_str(name);
    }
    #[inline]
    fn load(&mut self, name: &str, _ctx: &mut Context) {
        use std::fmt::Write;
        write!(self.0, "${}", name).unwrap();
    }
    #[inline]
    fn print(&mut self, v: Value) {
        use std::fmt::Write;
        write!(self.0, "{}", v).unwrap();
    }
    #[inline]
    fn new_line(&mut self) {
        self.0.push('@');
    }
    #[inline]
    fn wait(&mut self) {
        self.0.push('#');
    }
}
