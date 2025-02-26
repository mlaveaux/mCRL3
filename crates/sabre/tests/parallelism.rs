

#[test]
fn test_parallelism() {
    // let mut threads = vec![];

    // let test_case = (
    //     include_str!("../../../examples/REC/mcrl2/benchexpr10.dataspec"),
    //     include_str!("../../../examples/REC/mcrl2/benchexpr10.expressions"),
    //     include_str!("snapshot/result_benchexpr10.txt"),
    // );

    // for _ in 0..2 {
    //     threads.push(thread::spawn(move || {
    //         let (data_spec, expressions, expected_result) = test_case;

    //         let spec = DataSpecification::new(data_spec).unwrap();
    //         let terms: Vec<DataExpression> = expressions.lines().map(|text| spec.parse(text).unwrap()).collect();
    //         let mut expected = expected_result.split('\n');

    //         let mut inner = InnermostRewriter::new(&spec.clone().into());
    //         for term in &terms {
    //             let result = inner.rewrite(term.clone());

    //             let expected_result = spec.parse(expected.next().unwrap()).unwrap();
    //             assert_eq!(
    //                 result, expected_result,
    //                 "The inner rewrite result doesn't match the expected result"
    //             );
    //         }
    //     }));
    // }

    // for thread in threads {
    //     thread.join().unwrap();
    // }
}
