pub trait Printer {
    fn print(&mut self, text: &str);
    fn new_line(&mut self);
    fn wait(&mut self);
}

pub struct DummyPrinter;

impl Printer for DummyPrinter {
    fn print(&mut self, _: &str) {}
    fn new_line(&mut self) {}
    fn wait(&mut self) {}
}

pub struct RecordPrinter(String);

impl RecordPrinter {
    pub fn new() -> Self {
        Self(String::with_capacity(8196))
    }

    pub fn text(&self) -> &str {
        &self.0
    }
}

impl Printer for RecordPrinter {
    fn print(&mut self, text: &str) {
        self.0 += text;
    }
    fn new_line(&mut self) {
        self.0.push('\n');
    }
    fn wait(&mut self) {
        self.0.push('#');
    }
}
