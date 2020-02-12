use kes::parser::parse;
use bumpalo::Bump;
use std::env::args;

fn main() {
    if let Some(arg) = args().nth(1) {
        let bump =  Bump::with_capacity(8196);
        let code = std::fs::read_to_string(arg).unwrap();
        let instructions = parse(&bump, &code);
        println!("{:#?}", instructions);
    } else {
        println!("Usage: <program> <path>");
    }
}
