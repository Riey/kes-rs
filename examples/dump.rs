use kes::interner::Interner;
use kes::parser::parse;
use std::env::args;

fn main() {
    if let Some(arg) = args().nth(1) {
        let mut interner = Interner::default();
        let code = std::fs::read_to_string(arg).unwrap();
        let instructions = parse(&code, &mut interner).unwrap();
        println!("{:#?}", instructions);
    } else {
        println!("Usage: <program> <path>");
    }
}
