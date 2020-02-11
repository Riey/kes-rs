use crate::context::Value;

pub trait Printer {
    fn print(&mut self, v: Value);
    fn wait(&mut self);
}

impl<'a, P: Printer> Printer for &'a mut P {
    #[inline(always)]
    fn print(&mut self, v: Value) {
        (**self).print(v);
    }
    #[inline(always)]
    fn wait(&mut self) {
        (**self).wait();
    }
}

pub struct DummyPrinter;

impl Printer for DummyPrinter {
    fn print(&mut self, _: Value) {}
    fn wait(&mut self) {}
}

pub struct RecordPrinter(String);

impl RecordPrinter {
    #[inline(always)]
    pub fn new() -> Self {
        Self(String::with_capacity(8196))
    }

    #[inline(always)]
    pub fn text(&self) -> &str {
        &self.0
    }
}

impl Printer for RecordPrinter {
    #[inline(always)]
    fn print(&mut self, v: Value) {
        use std::fmt::Write;
        write!(self.0, "{}", v).unwrap();
    }
    #[inline(always)]
    fn wait(&mut self) {
        self.0.push('#');
    }
}
