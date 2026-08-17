use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

mod dedup_benchmarks;

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = dedup_benchmarks::bench_dedup_vs_transition_count,
        dedup_benchmarks::bench_dedup_vs_duplicate_ratio,
        dedup_benchmarks::bench_dedup_vs_num_states,
        dedup_benchmarks::bench_dedup_hub_state,
);
criterion_main!(benches);
