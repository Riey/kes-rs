#![feature(test)]

extern crate test;

use test::Bencher;

use kes::{compiler::compile, parser::parse};

#[bench]
pub fn parse_short(b: &mut Bencher) {
    let input = "@ 1 2 3 4 5 6 7 8 9 '1' '2' '3' '4' '5' '6' '7' '8' '9';";
    b.bytes += input.len() as u64;

    b.iter(|| {
        let insts = parse(&input).unwrap();
        assert!(!insts.is_empty());
    });
}

#[bench]
pub fn parse_long(b: &mut Bencher) {
    let input = "만약 1 + 2 == $1 { 123; } 그외 { 만약 $1 { } }".repeat(100);
    b.bytes += input.len() as u64;

    b.iter(|| {
        let insts = parse(&input).unwrap();
        assert!(!insts.is_empty());
    });
}

#[bench]
pub fn compile_long(b: &mut Bencher) {
    let input = "만약 1 + 2 == $1 { 123; } 그외 { 만약 $1 { } }".repeat(100);
    let ast = parse(&input).unwrap();
    b.bytes += input.len() as u64;

    b.iter(|| {
        let insts = compile(&ast);
        assert!(!insts.is_empty());
    });
}
