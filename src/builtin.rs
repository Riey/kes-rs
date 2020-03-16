use crate::bumpalo::Bump;
use crate::context::Context;
use crate::value::Value;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Builtin {
    async fn run<'c>(&mut self, name: &'_ str, ctx: &'_ mut Context<'c>);
    fn load<'b>(&mut self, name: &str, b: &'b Bump) -> Value<'b>;
    fn print(&mut self, v: Value);
    fn new_line(&mut self);
    async fn wait(&mut self);
}

#[async_trait(?Send)]
impl<'a, B: Builtin> Builtin for &'a mut B {
    #[inline]
    async fn run<'c>(&mut self, name: &'_ str, ctx: &'_ mut Context<'c>) {
        (**self).run(name, ctx).await;
    }
    #[inline]
    fn load<'b>(&mut self, name: &str, b: &'b Bump) -> Value<'b> {
        (**self).load(name, b)
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
    async fn wait(&mut self) {
        (**self).wait();
    }
}

pub struct DummyBuiltin;

#[async_trait(?Send)]
impl Builtin for DummyBuiltin {
    #[inline]
    async fn run<'c>(&mut self, _name: &'_ str, _ctx: &'_ mut Context<'c>) {}
    #[inline]
    fn load<'b>(&mut self, _name: &str, _b: &'b Bump) -> Value<'b> {
        Value::Int(0)
    }
    #[inline]
    fn print(&mut self, _v: Value) {}
    #[inline]
    fn new_line(&mut self) {}
    #[inline]
    async fn wait(&mut self) {}
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

#[async_trait(?Send)]
impl Builtin for RecordBuiltin {
    #[inline]
    async fn run<'c>(&mut self, name: &'_ str, _ctx: &'_ mut Context<'c>) {
        self.0.push_str(name);
    }
    #[inline]
    fn load<'b>(&mut self, name: &str, _b: &'b Bump) -> Value<'b> {
        use std::fmt::Write;
        write!(self.0, "${}", name).unwrap();
        Value::Int(0)
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
    async fn wait(&mut self) {
        self.0.push('#');
    }
}
