use crate::context::Context;
use crate::value::Value;
use async_trait::async_trait;

#[async_trait]
pub trait Builtin: Send {
    async fn run(&mut self, name: &str, ctx: &mut Context<'_>) -> Option<Value>;
    fn load<'b>(&mut self, name: &str) -> Value;
    fn print(&mut self, v: Value);
    fn new_line(&mut self);
    async fn wait(&mut self);
}

#[async_trait]
impl<'a, B: Builtin> Builtin for &'a mut B {
    #[inline]
    async fn run(&mut self, name: &str, ctx: &mut Context<'_>) -> Option<Value> {
        (**self).run(name, ctx).await
    }
    #[inline]
    fn load<'b>(&mut self, name: &str) -> Value {
        (**self).load(name)
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
        (**self).wait().await;
    }
}

#[cfg(test)]
pub struct RecordBuiltin(String);

#[cfg(test)]
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

#[cfg(test)]
#[async_trait]
impl Builtin for RecordBuiltin {
    #[inline]
    async fn run(&mut self, name: &str, _ctx: &mut Context<'_>) -> Option<Value> {
        self.0.push_str(name);
        None
    }
    #[inline]
    fn load<'b>(&mut self, name: &str) -> Value {
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
