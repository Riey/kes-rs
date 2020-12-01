use kes::parser::parse;
use std::env::args;

fn main() {
    if let Some(arg) = args().nth(1) {
        let code = std::fs::read_to_string(arg).unwrap();
        let instructions = parse(&code).unwrap();
        println!("{:#?}", instructions);
    } else {
        println!("Usage: <program> <path>");
    }
}
