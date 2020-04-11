use async_trait::async_trait;
use crate::value::Value;

#[async_trait(?Send)]
pub trait KesConsole {
    fn print(&mut self, v: Value);
    fn new_line(&mut self);
    async fn wait(&mut self);
}

#[async_trait(?Send)]
impl<'a, C: KesConsole> KesConsole for &'a mut C {
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
pub struct RecordConsole(pub String);

#[cfg(test)]
impl RecordConsole {
    pub fn new() -> Self {
        Self(String::with_capacity(8196))
    }
}

#[cfg(test)]
#[async_trait(?Send)]
impl KesConsole for RecordConsole {
    fn print(&mut self, v: Value) {
        self.0 += &v.to_string();
    }
    fn new_line(&mut self) {
        self.0.push('@');
    }
    async fn wait(&mut self) {
        self.0.push('#');
    }
}

