use mcrl2::data::DataExpression;
use std::cell::RefCell;
use std::rc::Rc;
use test_case::test_case;

use ahash::AHashSet;

use mcrl2::aterm::TermPool;
use rec_tests::load_REC_from_strings;
use sabre::utilities::to_untyped_data_expression;
use sabre::InnermostRewriter;
use sabre::RewriteEngine;
use sabre::RewriteSpecification;
use sabre::SabreRewriter;

#[test_case(vec![include_str!("../../../examples/REC/rec/benchexpr10.rec"), include_str!("../../../examples/REC/rec/asfsdfbenchmark.rec")], include_str!("snapshot/result_benchexpr10.txt") ; "benchexpr10")]
#[test_case(vec![include_str!("../../../examples/REC/rec/benchsym10.rec"), include_str!("../../../examples/REC/rec/asfsdfbenchmark.rec")], include_str!("snapshot/result_benchsym10.txt") ; "benchsym10")]
#[test_case(vec![include_str!("../../../examples/REC/rec/bubblesort10.rec"), include_str!("../../../examples/REC/rec/bubblesort.rec")], include_str!("snapshot/result_bubblesort10.txt") ; "bubblesort10")]
#[test_case(vec![include_str!("../../../examples/REC/rec/bubblesort20.rec"), include_str!("../../../examples/REC/rec/bubblesort.rec")], include_str!("snapshot/result_bubblesort20.txt") ; "bubblesort20")]
#[test_case(vec![include_str!("../../../examples/REC/rec/bubblesort100.rec"), include_str!("../../../examples/REC/rec/bubblesort.rec")], include_str!("snapshot/result_bubblesort100.txt") ; "bubblesort100")]
#[test_case(vec![include_str!("../../../examples/REC/rec/calls.rec")], include_str!("snapshot/result_calls.txt") ; "calls")]
#[test_case(vec![include_str!("../../../examples/REC/rec/check1.rec")], include_str!("snapshot/result_check1.txt") ; "check1")]
#[test_case(vec![include_str!("../../../examples/REC/rec/check2.rec")], include_str!("snapshot/result_check2.txt") ; "check2")]
#[test_case(vec![include_str!("../../../examples/REC/rec/confluence.rec")], include_str!("snapshot/result_confluence.txt") ; "confluence")]
#[test_case(vec![include_str!("../../../examples/REC/rec/factorial5.rec"), include_str!("../../../examples/REC/rec/factorial.rec")], include_str!("snapshot/result_factorial5.txt") ; "factorial5")]
#[test_case(vec![include_str!("../../../examples/REC/rec/factorial6.rec"), include_str!("../../../examples/REC/rec/factorial.rec")], include_str!("snapshot/result_factorial6.txt") ; "factorial6")]
#[test_case(vec![include_str!("../../../examples/REC/rec/fibonacci05.rec"), include_str!("../../../examples/REC/rec/fibonacci.rec")], include_str!("snapshot/result_fibonacci05.txt") ; "fibonacci05")]
#[test_case(vec![include_str!("../../../examples/REC/rec/garbagecollection.rec")], include_str!("snapshot/result_garbagecollection.txt") ; "garbagecollection")]
#[test_case(vec![include_str!("../../../examples/REC/rec/hanoi4.rec"), include_str!("../../../examples/REC/rec/hanoi.rec")], include_str!("snapshot/result_hanoi4.txt") ; "hanoi4")]
#[test_case(vec![include_str!("../../../examples/REC/rec/hanoi8.rec"), include_str!("../../../examples/REC/rec/hanoi.rec")], include_str!("snapshot/result_hanoi8.txt") ; "hanoi8")]
#[test_case(vec![include_str!("../../../examples/REC/rec/logic3.rec")], include_str!("snapshot/result_logic3.txt") ; "logic3")]
#[test_case(vec![include_str!("../../../examples/REC/rec/merge.rec")], include_str!("snapshot/result_merge.txt") ; "merge")]
#[test_case(vec![include_str!("../../../examples/REC/rec/mergesort10.rec"), include_str!("../../../examples/REC/rec/mergesort.rec")], include_str!("snapshot/result_mergesort10.txt") ; "mergesort10")]
#[test_case(vec![include_str!("../../../examples/REC/rec/missionaries2.rec"), include_str!("../../../examples/REC/rec/missionaries.rec")], include_str!("snapshot/result_missionaries2.txt") ; "missionaries2")]
#[test_case(vec![include_str!("../../../examples/REC/rec/missionaries3.rec"), include_str!("../../../examples/REC/rec/missionaries.rec")], include_str!("snapshot/result_missionaries3.txt") ; "missionaries3")]
#[test_case(vec![include_str!("../../../examples/REC/rec/quicksort10.rec"), include_str!("../../../examples/REC/rec/quicksort.rec")], include_str!("snapshot/result_quicksort10.txt") ; "quicksort10")]
#[test_case(vec![include_str!("../../../examples/REC/rec/revelt.rec")], include_str!("snapshot/result_revelt.txt") ; "revelt")]
#[test_case(vec![include_str!("../../../examples/REC/rec/revnat100.rec"), include_str!("../../../examples/REC/rec/revnat.rec")], include_str!("snapshot/result_revnat100.txt") ; "revnat100")]
#[test_case(vec![include_str!("../../../examples/REC/rec/searchinconditions.rec")], include_str!("snapshot/result_searchinconditions.txt") ; "searchinconditions")]
#[test_case(vec![include_str!("../../../examples/REC/rec/sieve20.rec"), include_str!("../../../examples/REC/rec/sieve.rec")], include_str!("snapshot/result_sieve20.txt") ; "sieve20")]
#[test_case(vec![include_str!("../../../examples/REC/rec/sieve100.rec"), include_str!("../../../examples/REC/rec/sieve.rec")], include_str!("snapshot/result_sieve100.txt") ; "sieve100")]
#[test_case(vec![include_str!("../../../examples/REC/rec/soundnessofparallelengines.rec")], include_str!("snapshot/result_soundnessofparallelengines.txt") ; "soundnessofparallelengines")]
#[test_case(vec![include_str!("../../../examples/REC/rec/tak18.rec"), include_str!("../../../examples/REC/rec/tak.rec")], include_str!("snapshot/result_tak18.txt") ; "tak18")]
#[test_case(vec![include_str!("../../../examples/REC/rec/tautologyhard.rec")], include_str!("snapshot/result_tautologyhard.txt") ; "tautologyhard")]
#[test_case(vec![include_str!("../../../examples/REC/rec/tricky.rec")], include_str!("snapshot/result_tricky.txt") ; "tricky")]
fn rec_test(rec_files: Vec<&str>, expected_result: &str) {
    let tp = Rc::new(RefCell::new(TermPool::new()));
    let (spec, terms): (RewriteSpecification, Vec<DataExpression>) = {
        let (syntax_spec, syntax_terms) = load_REC_from_strings(&mut tp.borrow_mut(), &rec_files).unwrap();
        let result = syntax_spec.to_rewrite_spec(&mut tp.borrow_mut());
        (
            result,
            syntax_terms
                .iter()
                .map(|t| to_untyped_data_expression(&mut tp.borrow_mut(), t, &AHashSet::new()))
                .collect(),
        )
    };

    // Test Sabre rewriter
    let mut sa = SabreRewriter::new(tp.clone(), &spec);
    let mut inner = InnermostRewriter::new(tp.clone(), &spec);

    let mut expected = expected_result.split('\n');

    for term in &terms {
        let expected_term = tp.borrow_mut().from_string(expected.next().unwrap()).unwrap();
        let expected_result = to_untyped_data_expression(&mut tp.borrow_mut(), &expected_term, &AHashSet::new());

        let result = inner.rewrite(term.clone());
        assert_eq!(
            result,
            expected_result.clone().into(),
            "The inner rewrite result doesn't match the expected result",
        );

        let result = sa.rewrite(term.clone());
        assert_eq!(
            result,
            expected_result.into(),
            "The sabre rewrite result doesn't match the expected result"
        );
    }
}

#[cfg(not(debug_assertions))]
#[test_case(vec![include_str!("../../../examples/REC/rec/benchexpr20.rec"), include_str!("../../../examples/REC/rec/asfsdfbenchmark.rec")], include_str!("snapshot/result_benchexpr20.txt") ; "benchexpr20")]
#[test_case(vec![include_str!("../../../examples/REC/rec/benchsym20.rec"), include_str!("../../../examples/REC/rec/asfsdfbenchmark.rec")], include_str!("snapshot/result_benchsym20.txt") ; "benchsym20")]
#[test_case(vec![include_str!("../../../examples/REC/rec/empty.rec")], include_str!("snapshot/result_empty.txt") ; "empty")]
#[test_case(vec![include_str!("../../../examples/REC/rec/evalexpr.rec")], include_str!("snapshot/result_evalexpr.txt") ; "evalexpr")]
#[test_case(vec![include_str!("../../../examples/REC/rec/evaltree.rec")], include_str!("snapshot/result_evaltree.txt") ; "evaltree")]
#[test_case(vec![include_str!("../../../examples/REC/rec/fibonacci18.rec"), include_str!("../../../examples/REC/rec/fibonacci.rec")], include_str!("snapshot/result_fibonacci18.txt") ; "fibonacci18")]
#[test_case(vec![include_str!("../../../examples/REC/rec/natlist.rec")], include_str!("snapshot/result_natlist.txt") ; "natlist")]
#[test_case(vec![include_str!("../../../examples/REC/rec/oddeven.rec")], include_str!("snapshot/result_oddeven.txt") ; "oddeven")]
#[test_case(vec![include_str!("../../../examples/REC/rec/order.rec")], include_str!("snapshot/result_order.txt") ; "order")]
#[test_case(vec![include_str!("../../../examples/REC/rec/permutations6.rec"), include_str!("../../../examples/REC/rec/permutations.rec")], include_str!("snapshot/result_permutations6.txt") ; "permutations6")]
fn rec_test_release(rec_files: Vec<&str>, expected_result: &str) {
    rec_test(rec_files, expected_result);
}

// These tests use more stack memory than is available on Windows.
#[cfg(all(unix, not(debug_assertions)))]
#[ignore]
#[test_case(vec![include_str!("../../../examples/REC/rec/sieve1000.rec"), include_str!("../../../examples/REC/rec/sieve.rec")], include_str!("snapshot/result_sieve1000.txt") ; "sieve1000")]
#[test_case(vec![include_str!("../../../examples/REC/rec/revnat1000.rec"), include_str!("../../../examples/REC/rec/revnat.rec")], include_str!("snapshot/result_revnat1000.txt") ; "revnat1000")]
#[test_case(vec![include_str!("../../../examples/REC/rec/closure.rec")], include_str!("snapshot/result_closure.txt") ; "closure")]
#[test_case(vec![include_str!("../../../examples/REC/rec/dart.rec")], include_str!("snapshot/result_dart.txt") ; "dart")]
#[test_case(vec![include_str!("../../../examples/REC/rec/factorial7.rec"), include_str!("../../../examples/REC/rec/factorial.rec")], include_str!("snapshot/result_factorial7.txt") ; "factorial7")]
#[test_case(vec![include_str!("../../../examples/REC/rec/factorial8.rec"), include_str!("../../../examples/REC/rec/factorial.rec")], include_str!("snapshot/result_factorial8.txt") ; "factorial8")]
// #[test_case(vec![include_str!("../../../examples/REC/rec/factorial9.rec"), include_str!("../../../examples/REC/rec/factorial.rec")], include_str!("snapshot/result_factorial9.txt") ; "factorial9")]
#[test_case(vec![include_str!("../../../examples/REC/rec/fibonacci19.rec"), include_str!("../../../examples/REC/rec/fibonacci.rec")], include_str!("snapshot/result_fibonacci19.txt") ; "fibonacci19")]
#[test_case(vec![include_str!("../../../examples/REC/rec/fibonacci20.rec"), include_str!("../../../examples/REC/rec/fibonacci.rec")], include_str!("snapshot/result_fibonacci20.txt") ; "fibonacci20")]
#[test_case(vec![include_str!("../../../examples/REC/rec/fibonacci21.rec"), include_str!("../../../examples/REC/rec/fibonacci.rec")], include_str!("snapshot/result_fibonacci21.txt") ; "fibonacci21")]
#[test_case(vec![include_str!("../../../examples/REC/rec/hanoi12.rec"), include_str!("../../../examples/REC/rec/hanoi.rec")], include_str!("snapshot/result_hanoi12.txt") ; "hanoi12")]
#[test_case(vec![include_str!("../../../examples/REC/rec/permutations7.rec"), include_str!("../../../examples/REC/rec/permutations.rec")], include_str!("snapshot/result_permutations7.txt") ; "permutations7")]
fn rec_test_unix(rec_files: Vec<&str>, expected_result: &str) {
    rec_test(rec_files, expected_result);
}
