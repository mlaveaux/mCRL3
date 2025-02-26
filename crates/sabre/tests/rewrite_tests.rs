
use test_case::test_case;


#[test_case(include_str!("../../../examples/REC/mcrl2/benchexpr10.dataspec"), include_str!("../../../examples/REC/mcrl2/benchexpr10.expressions"), include_str!("snapshot/result_benchexpr10.txt") ; "benchexpr10")]
#[test_case(include_str!("../../../examples/REC/mcrl2/benchsym10.dataspec"), include_str!("../../../examples/REC/mcrl2/benchsym10.expressions"), include_str!("snapshot/result_benchsym10.txt") ; "benchsym10")]
#[test_case(include_str!("../../../examples/REC/mcrl2/calls.dataspec"), include_str!("../../../examples/REC/mcrl2/calls.expressions"), include_str!("snapshot/result_calls.txt") ; "calls")]
#[test_case(include_str!("../../../examples/REC/mcrl2/check1.dataspec"), include_str!("../../../examples/REC/mcrl2/check1.expressions"), include_str!("snapshot/result_check1.txt") ; "check1")]
#[test_case(include_str!("../../../examples/REC/mcrl2/check2.dataspec"), include_str!("../../../examples/REC/mcrl2/check2.expressions"), include_str!("snapshot/result_check2.txt") ; "check2")]
#[test_case(include_str!("../../../examples/REC/mcrl2/confluence.dataspec"), include_str!("../../../examples/REC/mcrl2/confluence.expressions"), include_str!("snapshot/result_confluence.txt") ; "confluence")]
#[test_case(include_str!("../../../examples/REC/mcrl2/fibonacci05.dataspec"), include_str!("../../../examples/REC/mcrl2/fibonacci05.expressions"), include_str!("snapshot/result_fibonacci05.txt") ; "fibonacci05")]
#[test_case(include_str!("../../../examples/REC/mcrl2/garbagecollection.dataspec"), include_str!("../../../examples/REC/mcrl2/garbagecollection.expressions"), include_str!("snapshot/result_garbagecollection.txt") ; "garbagecollection")]
#[test_case(include_str!("../../../examples/REC/mcrl2/logic3.dataspec"), include_str!("../../../examples/REC/mcrl2/logic3.expressions"), include_str!("snapshot/result_logic3.txt") ; "logic3")]
#[test_case(include_str!("../../../examples/REC/mcrl2/merge.dataspec"), include_str!("../../../examples/REC/mcrl2/merge.expressions"), include_str!("snapshot/result_merge.txt") ; "merge")]
#[test_case(include_str!("../../../examples/REC/mcrl2/mergesort10.dataspec"), include_str!("../../../examples/REC/mcrl2/mergesort10.expressions"), include_str!("snapshot/result_mergesort10.txt") ; "mergesort10")]
#[test_case(include_str!("../../../examples/REC/mcrl2/missionaries2.dataspec"), include_str!("../../../examples/REC/mcrl2/missionaries2.expressions"), include_str!("snapshot/result_missionaries2.txt") ; "missionaries2")]
#[test_case(include_str!("../../../examples/REC/mcrl2/missionaries3.dataspec"), include_str!("../../../examples/REC/mcrl2/missionaries3.expressions"), include_str!("snapshot/result_missionaries3.txt") ; "missionaries3")]
#[test_case(include_str!("../../../examples/REC/mcrl2/quicksort10.dataspec"), include_str!("../../../examples/REC/mcrl2/quicksort10.expressions"), include_str!("snapshot/result_quicksort10.txt") ; "quicksort10")]
#[test_case(include_str!("../../../examples/REC/mcrl2/revelt.dataspec"), include_str!("../../../examples/REC/mcrl2/revelt.expressions"), include_str!("snapshot/result_revelt.txt") ; "revelt")]
#[test_case(include_str!("../../../examples/REC/mcrl2/searchinconditions.dataspec"), include_str!("../../../examples/REC/mcrl2/searchinconditions.expressions"), include_str!("snapshot/result_searchinconditions.txt") ; "searchinconditions")]
#[test_case(include_str!("../../../examples/REC/mcrl2/soundnessofparallelengines.dataspec"), include_str!("../../../examples/REC/mcrl2/soundnessofparallelengines.expressions"), include_str!("snapshot/result_soundnessofparallelengines.txt") ; "soundnessofparallelengines")]
#[test_case(include_str!("../../../examples/REC/mcrl2/tautologyhard.dataspec"), include_str!("../../../examples/REC/mcrl2/tautologyhard.expressions"), include_str!("snapshot/result_tautologyhard.txt") ; "tautologyhard")]
fn rewriter_test(data_spec: &str, expressions: &str, expected_result: &str) {
    let _ = env_logger::builder().is_test(true).try_init();

    // let spec = DataSpecification::new(data_spec).unwrap();
    // let terms: Vec<DataExpression> = expressions.lines().map(|text| spec.parse(text).unwrap()).collect();

    // let mut inner = InnermostRewriter::new(&spec.clone().into());
    // let mut expected = expected_result.split('\n');

    // for term in &terms {
    //     let expected_result = spec.parse(expected.next().unwrap()).unwrap();

    //     let result = inner.rewrite(term.clone());
    //     assert_eq!(
    //         result, expected_result,
    //         "The inner rewrite result doesn't match the expected result"
    //     );

    //     // let result = sa.rewrite(term.clone());
    //     // assert_eq!(result, expected_result, "The sabre rewrite result doesn't match the expected result");
    // }
}

#[cfg(not(debug_assertions))]
#[test_case(include_str!("../../../examples/REC/mcrl2/benchexpr20.dataspec"), include_str!("../../../examples/REC/mcrl2/benchexpr20.expressions"), include_str!("snapshot/result_benchexpr20.txt") ; "benchexpr20")]
#[test_case(include_str!("../../../examples/REC/mcrl2/benchsym20.dataspec"), include_str!("../../../examples/REC/mcrl2/benchsym20.expressions"), include_str!("snapshot/result_benchsym20.txt") ; "benchsym20")]
#[test_case(include_str!("../../../examples/REC/mcrl2/closure.dataspec"), include_str!("../../../examples/REC/mcrl2/closure.expressions"), include_str!("snapshot/result_closure.txt") ; "closure")]
#[test_case(include_str!("../../../examples/REC/mcrl2/empty.dataspec"), include_str!("../../../examples/REC/mcrl2/empty.expressions"), include_str!("snapshot/result_empty.txt") ; "empty")]
#[test_case(include_str!("../../../examples/REC/mcrl2/evalexpr.dataspec"), include_str!("../../../examples/REC/mcrl2/evalexpr.expressions"), include_str!("snapshot/result_evalexpr.txt") ; "evalexpr")]
#[test_case(include_str!("../../../examples/REC/mcrl2/evaltree.dataspec"), include_str!("../../../examples/REC/mcrl2/evaltree.expressions"), include_str!("snapshot/result_evaltree.txt") ; "evaltree")]
#[test_case(include_str!("../../../examples/REC/mcrl2/oddeven.dataspec"), include_str!("../../../examples/REC/mcrl2/oddeven.expressions"), include_str!("snapshot/result_oddeven.txt") ; "oddeven")]
#[test_case(include_str!("../../../examples/REC/mcrl2/order.dataspec"), include_str!("../../../examples/REC/mcrl2/order.expressions"), include_str!("snapshot/result_order.txt") ; "order")]
#[test_case(include_str!("../../../examples/REC/mcrl2/revnat100.dataspec"), include_str!("../../../examples/REC/mcrl2/revnat100.expressions"), include_str!("snapshot/result_revnat100.txt") ; "revnat100")]
#[test_case(include_str!("../../../examples/REC/mcrl2/sieve20.dataspec"), include_str!("../../../examples/REC/mcrl2/sieve20.expressions"), include_str!("snapshot/result_sieve20.txt") ; "sieve20")]
#[test_case(include_str!("../../../examples/REC/mcrl2/sieve100.dataspec"), include_str!("../../../examples/REC/mcrl2/sieve100.expressions"), include_str!("snapshot/result_sieve100.txt") ; "sieve100")]
fn rewriter_test_release(data_spec: &str, expressions: &str, expected_result: &str) {
    rewriter_test(data_spec, expressions, expected_result);
}

#[cfg(unix)]
#[cfg(not(debug_assertions))]
// #[test_case(include_str!("../../../examples/REC/mcrl2/add8.dataspec"), include_str!("../../../examples/REC/mcrl2/add8.expressions"), include_str!("snapshot/result_add8.txt") ; "add8")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/add16.dataspec"), include_str!("../../../examples/REC/mcrl2/add16.expressions"), include_str!("snapshot/result_add16.txt") ; "add16")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/add32.dataspec"), include_str!("../../../examples/REC/mcrl2/add32.expressions"), include_str!("snapshot/result_add32.txt") ; "add32")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/bubblesort10.dataspec"), include_str!("../../../examples/REC/mcrl2/bubblesort10.expressions"), include_str!("snapshot/result_bubblesort10.txt") ; "bubblesort10")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/bubblesort20.dataspec"), include_str!("../../../examples/REC/mcrl2/bubblesort20.expressions"), include_str!("snapshot/result_bubblesort20.txt") ; "bubblesort20")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/bubblesort100.dataspec"), include_str!("../../../examples/REC/mcrl2/bubblesort100.expressions"), include_str!("snapshot/result_bubblesort100.txt") ; "bubblesort100")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/dart.dataspec"), include_str!("../../../examples/REC/mcrl2/dart.expressions"), include_str!("snapshot/result_dart.txt") ; "dart")]
// #[test_cas//e(include_str!("../../../examples/REC/mcrl2/factorial5.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial5.expressions"), include_str!("snapshot/result_factorial5.txt") ; "factorial5")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/factorial6.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial6.expressions"), include_str!("snapshot/result_factorial6.txt") ; "factorial6")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/factorial7.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial7.expressions"), include_str!("snapshot/result_factorial7.txt") ; "factorial7")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/factorial8.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial8.expressions"), include_str!("snapshot/result_factorial8.txt") ; "factorial8")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/factorial9.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial9.expressions"), include_str!("snapshot/result_factorial9.txt") ; "factorial9")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/fibonacci18.dataspec"), include_str!("../../../examples/REC/mcrl2/fibonacci18.expressions"), include_str!("snapshot/result_fibonacci18.txt") ; "fibonacci18")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/fibonacci19.dataspec"), include_str!("../../../examples/REC/mcrl2/fibonacci19.expressions"), include_str!("snapshot/result_fibonacci19.txt") ; "fibonacci19")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/fibonacci20.dataspec"), include_str!("../../../examples/REC/mcrl2/fibonacci20.expressions"), include_str!("snapshot/result_fibonacci20.txt") ; "fibonacci20")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/fibonacci21.dataspec"), include_str!("../../../examples/REC/mcrl2/fibonacci21.expressions"), include_str!("snapshot/result_fibonacci21.txt") ; "fibonacci21")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/hanoi4.dataspec"), include_str!("../../../examples/REC/mcrl2/hanoi4.expressions"), include_str!("snapshot/result_hanoi4.txt") ; "hanoi4")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/hanoi8.dataspec"), include_str!("../../../examples/REC/mcrl2/hanoi8.expressions"), include_str!("snapshot/result_hanoi8.txt") ; "hanoi8")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/hanoi12.dataspec"), include_str!("../../../examples/REC/mcrl2/hanoi12.expressions"), include_str!("snapshot/result_hanoi12.txt") ; "hanoi12")]
#[test_case(include_str!("../../../examples/REC/mcrl2/permutations6.dataspec"), include_str!("../../../examples/REC/mcrl2/permutations6.expressions"), include_str!("snapshot/result_permutations6.txt") ; "permutations6")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/permutations7.dataspec"), include_str!("../../../examples/REC/mcrl2/permutations7.expressions"), include_str!("snapshot/result_permutations7.txt") ; "permutations7")]
#[test_case(include_str!("../../../examples/REC/mcrl2/natlist.dataspec"), include_str!("../../../examples/REC/mcrl2/natlist.expressions"), include_str!("snapshot/result_natlist.txt") ; "natlist")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/revnat1000.dataspec"), include_str!("../../../examples/REC/mcrl2/revnat1000.expressions"), include_str!("snapshot/result_revnat1000.txt") ; "revnat1000")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/sieve1000.dataspec"), include_str!("../../../examples/REC/mcrl2/sieve1000.expressions"), include_str!("snapshot/result_sieve1000.txt") ; "sieve1000")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/tak18.dataspec"), include_str!("../../../examples/REC/mcrl2/tak18.expressions"), include_str!("snapshot/result_tak18.txt") ; "tak18")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/tricky.dataspec"), include_str!("../../../examples/REC/mcrl2/tricky.expressions"), include_str!("snapshot/result_tricky.txt") ; "tricky")]
fn rewriter_test_release_unix(data_spec: &str, expressions: &str, expected_result: &str) {
    rewriter_test(data_spec, expressions, expected_result);
}
