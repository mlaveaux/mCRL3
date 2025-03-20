use std::hint::black_box;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mcrl3_rec_tests::load_REC_from_strings;
use mcrl3_sabre::set_automaton::SetAutomaton;

pub fn criterion_benchmark_set_automaton(c: &mut Criterion) {
    for (name, rec_files) in [("hanoi8", [include_str!("../../../../examples/REC/rec/fibfree.rec")])] {
        let (syntax_spec, _) = load_REC_from_strings(&rec_files).unwrap();
        let result = syntax_spec.to_rewrite_spec();

        c.bench_function(&format!("set automaton {}", name), |bencher| {
            bencher.iter(|| {
                let _ = black_box(SetAutomaton::new(&result, |_| (), false));
            });
        });

        c.bench_function(&format!("apma automaton {}", name), |bencher| {
            bencher.iter(|| {
                let _ = black_box(SetAutomaton::new(&result, |_| (), true));
            });
        });
    }
}

criterion_group!(benches, criterion_benchmark_set_automaton,);
criterion_main!(benches);
