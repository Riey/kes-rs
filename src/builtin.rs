use crate::bumpalo::Bump;
use crate::console::KesConsole;
use crate::context::Context;
use crate::value::Value;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Builtin {
    async fn run<'c, C: KesConsole>(&mut self, name: &'_ str, ctx: &'_ mut Context<'c>, console: &mut C) -> Option<Value<'c>>;
    fn load<'b>(&mut self, name: &str, b: &'b Bump) -> Value<'b>;
}

#[async_trait(?Send)]
impl<'a, B: Builtin> Builtin for &'a mut B {
    #[inline]
    async fn run<'c, C: KesConsole>(&mut self, name: &'_ str, ctx: &'_ mut Context<'c>, console: &mut C) -> Option<Value<'c>> {
        (**self).run(name, ctx, console).await
    }
    #[inline]
    fn load<'b>(&mut self, name: &str, b: &'b Bump) -> Value<'b> {
        (**self).load(name, b)
    }
}

pub struct DummyBuiltin;

#[async_trait(?Send)]
impl<'a> Builtin for DummyBuiltin {
    #[inline]
    async fn run<'c, C: KesConsole>(&mut self, _name: &'_ str, _ctx: &'_ mut Context<'c>, _console: &mut C) -> Option<Value<'c>> {
        None
    }
    #[inline]
    fn load<'b>(&mut self, _name: &str, _b: &'b Bump) -> Value<'b> {
        Value::Int(0)
    }
}

#[cfg(test)]
pub struct RecordBuiltin(pub String);

#[cfg(test)]
impl RecordBuiltin {
    #[inline]
    pub fn new() -> Self {
        Self(String::with_capacity(8196))
    }
}

#[cfg(test)]
#[async_trait(?Send)]
impl Builtin for RecordBuiltin {
    #[inline]
    async fn run<'c, C: KesConsole>(&'_ mut self, name: &'_ str, _ctx: &'_ mut Context<'c>, _console: &'_ mut C) -> Option<Value<'c>> {
        self.0.push_str(name);
        None
    }
    #[inline]
    fn load<'b>(&mut self, name: &str, _b: &'b Bump) -> Value<'b> {
        self.0.push('$');
        self.0.push_str(name);
        Value::Int(0)
    }
}
