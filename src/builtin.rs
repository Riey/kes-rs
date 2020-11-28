use crate::context::Context;
use crate::error::RuntimeResult;
use crate::value::Value;

pub trait Builtin {
    fn run<'c>(&mut self, name: &'c str, ctx: &mut Context<'c>)
        -> RuntimeResult<Option<Value<'c>>>;
    fn load<'c>(&mut self, name: &'c str, ctx: &mut Context<'c>) -> RuntimeResult<Value<'c>>;
    fn print(&mut self, v: Value);
    fn new_line(&mut self);
}

impl<'a, B: Builtin> Builtin for &'a mut B {
    #[inline]
    fn run<'c>(
        &mut self,
        name: &'c str,
        ctx: &mut Context<'c>,
    ) -> RuntimeResult<Option<Value<'c>>> {
        (**self).run(name, ctx)
    }
    #[inline]
    fn load<'c>(&mut self, name: &'c str, ctx: &mut Context<'c>) -> RuntimeResult<Value<'c>> {
        (**self).load(name, ctx)
    }
    #[inline]
    fn print(&mut self, v: Value) {
        (**self).print(v);
    }
    #[inline]
    fn new_line(&mut self) {
        (**self).new_line();
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
impl Builtin for RecordBuiltin {
    #[inline]
    fn run<'c>(
        &mut self,
        name: &'c str,
        _ctx: &mut Context<'c>,
    ) -> RuntimeResult<Option<Value<'c>>> {
        self.0.push_str(name);
        Ok(None)
    }
    #[inline]
    fn load<'c>(&mut self, name: &'c str, _ctx: &mut Context<'c>) -> RuntimeResult<Value<'c>> {
        use std::fmt::Write;
        write!(self.0, "${}", name).unwrap();
        Ok(Value::Int(0))
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
}
