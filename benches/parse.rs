#![feature(test)]

extern crate test;

use kes::interner::Interner;
use kes::parser::parse;
use kes::program::Program;
use test::Bencher;

fn get_long_code() -> String {
    "만약 1 + 2 == $1 { 123; } 그외 { 만약 $1 { } }".repeat(100)
}

#[bench]
pub fn parse_short(b: &mut Bencher) {
    let input = "@ 1 2 3 4 5 6 7 8 9 '1' '2' '3' '4' '5' '6' '7' '8' '9';";
    let mut interner = Interner::new();
    b.bytes += input.len() as u64;

    b.iter(|| {
        let insts = parse(&input, &mut interner).unwrap();
        assert!(!insts.is_empty());
    });
}

#[bench]
pub fn parse_long(b: &mut Bencher) {
    let input = get_long_code();
    let mut interner = Interner::new();
    b.bytes += input.len() as u64;

    b.iter(|| {
        let insts = parse(&input, &mut interner).unwrap();
        assert!(!insts.is_empty());
    });
}

#[bench]
pub fn compile_ast_long(b: &mut Bencher) {
    let input = get_long_code();
    let mut interner = Interner::new();
    b.bytes += input.len() as u64;

    let ast = parse(&input, &mut interner).unwrap();

    b.iter(|| {
        let program = Program::from_ast(&ast, interner.clone());
        assert!(!program.instructions().is_empty());
    });
}

#[bench]
pub fn deserialize_bytecode_long(b: &mut Bencher) {
    let input = get_long_code();
    b.bytes += input.len() as u64;

    let program = Program::from_source(&input).unwrap();

    let bytes = bincode::serialize(&program).unwrap();

    b.iter(|| {
        let program: Program = bincode::deserialize(&bytes).unwrap();
        assert!(!program.instructions().is_empty());
    })
}

#[bench]
pub fn format_long(b: &mut Bencher) {
    let input = get_long_code();
    b.bytes += input.len() as u64;

    let mut out = Vec::new();

    b.iter(|| {
        kes::formatter::format_code(&input, &mut out).unwrap();
        out.clear();
    })
}
