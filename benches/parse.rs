use criterion::*;
use kes::parser::parse;
use kes::lexer::lex;

pub fn throughput_short_bench(c: &mut Criterion) {
    let input = "1 2 3 4 5 6 7 8 9 '1' '2' '3' '4' '5' '6' '7' '8' '9' 정리";
    let mut group = c.benchmark_group("throughput-short");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input("lex", &input, |b, i| {
        b.iter(|| {
            assert_eq!(lex(i).count(), 19);
        });
    });
    group.bench_with_input("parse", &input, |b, i| {
        b.iter(|| {
            let insts = parse(lex(i));
            assert!(!insts.is_empty());
        });
    });
}

pub fn throughput_long_bench(c: &mut Criterion) {
    let input = "1 2 + -> $1 $1 { 123 } 그외 { 선택 $1 { 2 { } 그외 { } } }".repeat(100);
    let mut group = c.benchmark_group("throughput-long");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input("lex", &input, |b, i| {
        b.iter(|| {
            assert_eq!(lex(i).count(), 2200);
        });
    });
    group.bench_with_input("parse", &input, |b, i| {
        b.iter(|| {
            let insts = parse(lex(i));
            assert!(!insts.is_empty());
        });
    });
}

criterion_group!(benches, throughput_short_bench, throughput_long_bench);
criterion_main!(benches);

