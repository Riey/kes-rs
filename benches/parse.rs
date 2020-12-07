#![feature(test)]

extern crate test;

use kes::program::Program;
use test::Bencher;

#[bench]
pub fn compile_long(b: &mut Bencher) {
    let input = "만약 1 + 2 == $1 { 123; } 그외 { 만약 $1 { } }".repeat(100);
    b.bytes += input.len() as u64;

    b.iter(|| {
        let program = Program::from_source(&input).unwrap();
        assert!(!program.instructions().is_empty());
    });
}

#[bench]
pub fn deserialize_bytecode_long(b: &mut Bencher) {
    let input = "만약 1 + 2 == $1 { 123; } 그외 { 만약 $1 { } }".repeat(100);
    b.bytes += input.len() as u64;

    let program = Program::from_source(&input).unwrap();

    let bytes = bincode::serialize(&program).unwrap();

    b.iter(|| {
        let program: Program = bincode::deserialize(&bytes).unwrap();
        assert!(!program.instructions().is_empty());
    })
}
