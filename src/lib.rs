#![feature(str_strip)]

mod ast;
mod lexer;
mod token;

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Int(u32),
    Str(String),
}

//pub struct Context {
//    vars: BTreeMap<String, Value>,
//    builtin_functions: BTreeMap<String, fn(&mut Self)>,
//    stack: Vec<Value>,
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
