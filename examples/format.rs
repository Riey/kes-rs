use std::env;
use std::io::Write;

fn main() {
    if let Some(arg) = env::args().nth(1) {
        let code = std::fs::read_to_string(arg).unwrap();
        let out = std::io::stdout();
        let mut out = out.lock();
        kes::formatter::format_code(&code, &mut out).unwrap();
        out.flush().unwrap();
    } else {
        println!("Usage: <program> <path>");
    }
}
