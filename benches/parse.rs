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
            let insts: Vec<_> = parse(lex(i));
            assert!(!insts.is_empty());
        });
    });
}

criterion_group!(benches, throughput_short_bench);
criterion_main!(benches);

