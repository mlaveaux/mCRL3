use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

use mcrl2::aterm::TermPool;
use mcrl2::data::DataExpression;
use mcrl2::data::DataSpecification;
use sabre::InnermostRewriter;
use sabre::RewriteEngine;

#[test]
fn test_parallelism() {
    let mut threads = vec![];

    let test_case = (
        include_str!("../../../examples/REC/mcrl2/benchexpr10.dataspec"),
        include_str!("../../../examples/REC/mcrl2/benchexpr10.expressions"),
        include_str!("snapshot/result_benchexpr10.txt"),
    );

    for _ in 0..2 {
        threads.push(thread::spawn(move || {
            let (data_spec, expressions, expected_result) = test_case;

            let tp = Rc::new(RefCell::new(TermPool::new()));
            let spec = DataSpecification::new(data_spec).unwrap();
            let terms: Vec<DataExpression> = expressions.lines().map(|text| spec.parse(text).unwrap()).collect();
            let mut expected = expected_result.split('\n');

            let mut inner = InnermostRewriter::new(tp.clone(), &spec.clone().into());
            for term in &terms {
                let result = inner.rewrite(term.clone());

                let expected_result = spec.parse(expected.next().unwrap()).unwrap();
                assert_eq!(
                    result, expected_result,
                    "The inner rewrite result doesn't match the expected result"
                );
            }
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
