use kes::program::Program;
use std::env::args;

fn main() {
    if let Some(arg) = args().nth(1) {
        let code = std::fs::read_to_string(arg).unwrap();
        let program = Program::from_source(&code).unwrap();
        println!("{}", serde_json::to_string(&program).unwrap());
    } else {
        println!("Usage: <program> <path>");
    }
}
