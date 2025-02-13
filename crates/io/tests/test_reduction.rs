use mcrl3_reduction::branching_bisim_sigref;
use mcrl3_reduction::branching_bisim_sigref_naive;
use mcrl3_reduction::strong_bisim_sigref;
use mcrl3_reduction::strong_bisim_sigref_naive;
use mcrl3_utilities::Timing;
use test_case::test_case;

use mcrl3_io::io_aut::read_aut;

#[test_case(include_str!("../../../examples/lts/abp.aut") ; "abp.aut")]
#[test_case(include_str!("../../../examples/lts/cwi_1_2.aut") ; "cwi_1_2.aut")]
#[test_case(include_str!("../../../examples/lts/cwi_3_14.aut") ; "cwi_3_14.aut")]
#[test_case(include_str!("../../../examples/lts/selfloops.aut") ; "selfloops.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_0_1.aut") ; "vasy_0_1.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_1_4.aut") ; "vasy_1_4.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_5_9.aut") ; "vasy_5_9.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_8_24.aut") ; "vasy_8_24.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_25_25.aut") ; "vasy_25_25.aut")]
fn test_strong_bisimilation_reduction(input: &str) {
    let _ = env_logger::builder().is_test(true).try_init();

    let lts = read_aut(input.as_bytes(), vec![]).unwrap();
    let mut timing = Timing::new();

    let reduced = strong_bisim_sigref(&lts, &mut timing);
    let naive_reduced = strong_bisim_sigref_naive(&lts, &mut timing);

    assert_eq!(reduced, naive_reduced, "The partitions are not equal");
}

#[test_case(include_str!("../../../examples/lts/abp.aut") ; "abp.aut")]
#[test_case(include_str!("../../../examples/lts/cwi_1_2.aut") ; "cwi_1_2.aut")]
#[test_case(include_str!("../../../examples/lts/cwi_3_14.aut") ; "cwi_3_14.aut")]
#[test_case(include_str!("../../../examples/lts/selfloops.aut") ; "selfloops.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_0_1.aut") ; "vasy_0_1.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_1_4.aut") ; "vasy_1_4.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_5_9.aut") ; "vasy_5_9.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_8_24.aut") ; "vasy_8_24.aut")]
#[test_case(include_str!("../../../examples/lts/vasy_25_25.aut") ; "vasy_25_25.aut")]
fn test_branching_bisimilation_reduction(input: &str) {
    let _ = env_logger::builder().is_test(true).try_init();

    let lts = read_aut(input.as_bytes(), vec!["tau".into(), "i".into()]).unwrap();
    let mut timing = Timing::new();

    let reduced = branching_bisim_sigref(&lts, &mut timing);
    let naive_reduced = branching_bisim_sigref_naive(&lts, &mut timing);

    assert_eq!(reduced, naive_reduced, "The partitions are not equal");
}
