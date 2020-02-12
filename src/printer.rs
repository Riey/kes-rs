use crate::value::Value;

pub trait Printer {
    fn print(&mut self, v: Value);
    fn new_line(&mut self);
    fn wait(&mut self);
}

impl<'a, P: Printer> Printer for &'a mut P {
    #[inline(always)]
    fn print(&mut self, v: Value) {
        (**self).print(v);
    }
    #[inline(always)]
    fn new_line(&mut self) {
        (**self).new_line();
    }
    #[inline(always)]
    fn wait(&mut self) {
        (**self).wait();
    }
}

pub struct DummyPrinter;

impl Printer for DummyPrinter {
    #[inline(always)]
    fn print(&mut self, _: Value) {}
    #[inline(always)]
    fn new_line(&mut self) {}
    #[inline(always)]
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
    fn new_line(&mut self) {
        self.0.push('@');
    }
    #[inline(always)]
    fn wait(&mut self) {
        self.0.push('#');
    }
}
