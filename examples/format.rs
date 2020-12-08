use std::env;

fn main() {
    if let Some(arg) = env::args().nth(1) {
        let code = std::fs::read_to_string(arg).unwrap();
        kes::formatter::format_code(&code, std::io::stdout().lock()).unwrap();
    } else {
        println!("Usage: <program> <path>");
    }
}
