use std::cell::RefCell;
use std::hint::black_box;
use std::rc::Rc;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use mcrl2::aterm::TermPool;
use mcrl2::data::DataExpression;
use mcrl2::data::DataSpecification;
use mcrl2::data::JittyRewriter;
use rec_tests::load_REC_from_strings;
use sabre::set_automaton::SetAutomaton;
use sabre::InnermostRewriter;
use sabre::RewriteEngine;

/// Creates a rewriter and a vector of ATerm expressions for the given case.
pub fn load_case(
    _: &mut TermPool,
    data_spec_text: &str,
    expressions_text: &str,
    max_number_expressions: usize,
) -> (DataSpecification, Vec<DataExpression>) {
    // Read the data specification
    let data_spec = DataSpecification::new(&data_spec_text).unwrap();

    // Read the file line by line, and return an iterator of the lines of the file.
    let expressions: Vec<DataExpression> = expressions_text
        .lines()
        .take(max_number_expressions)
        .map(|x| data_spec.parse(x).unwrap())
        .collect();

    (data_spec, expressions)
}

pub fn criterion_benchmark_jitty(c: &mut Criterion) {
    for (name, data_spec, expressions) in [(
        "add8",
        include_str!("../../../../examples/REC/mcrl2/add8.dataspec"),
        include_str!("../../../../examples/REC/mcrl2/add8.expressions"),
    )] {
        let tp = Rc::new(RefCell::new(TermPool::new()));
        let (data_spec, expressions) = load_case(&mut tp.borrow_mut(), data_spec, expressions, 1);

        let mut jitty = JittyRewriter::new(&data_spec);
        let mut inner = InnermostRewriter::new(tp.clone(), &data_spec.into());

        c.bench_function(&format!("innermost {}", name), |bencher| {
            bencher.iter(|| {
                let _ = black_box(inner.rewrite(expressions[0].clone()));
            })
        });

        c.bench_function(&format!("jitty {}", name), |bencher| {
            bencher.iter(|| {
                let _ = black_box(jitty.rewrite(expressions[0].clone()));
            })
        });
    }
}

pub fn criterion_benchmark_set_automaton(c: &mut Criterion) {
    for (name, rec_files) in [("hanoi8", [include_str!("../../../../examples/REC/rec/fibfree.rec")])] {
        let tp = Rc::new(RefCell::new(TermPool::new()));
        let (syntax_spec, _) = load_REC_from_strings(&mut tp.borrow_mut(), &rec_files).unwrap();
        let result = syntax_spec.to_rewrite_spec(&mut tp.borrow_mut());

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

criterion_group!(benches, criterion_benchmark_jitty, criterion_benchmark_set_automaton,);
criterion_main!(benches);
