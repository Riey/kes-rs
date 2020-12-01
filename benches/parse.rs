#![feature(test)]

extern crate test;

use test::Bencher;

use kes::parser::parse;

#[bench]
pub fn parse_short(b: &mut Bencher) {
    let input = "1 2 3 4 5 6 7 8 9 '1' '2' '3' '4' '5' '6' '7' '8' '9' 정리";
    b.bytes += input.len() as u64;

    b.iter(|| {
        let insts = parse(&input).unwrap();
        assert!(!insts.is_empty());
    });
}

#[bench]
pub fn parse_long(b: &mut Bencher) {
    let input = "만약 1 2 + [$1] $1 { 123 } 그외 { 선택 $1 { 2 { } _ { } } }".repeat(100);
    b.bytes += input.len() as u64;

    b.iter(|| {
        let insts = parse(&input).unwrap();
        assert!(!insts.is_empty());
    });
}
