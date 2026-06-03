use criterion::{criterion_group, criterion_main, Criterion};
use parallel_revm_lab_executor::{run_access_list, run_optimistic, run_sequential};
use parallel_revm_lab_workload::{generate_workload, WorkloadConfig, WorkloadKind};

fn executor_benchmarks(c: &mut Criterion) {
    let workload = generate_workload(WorkloadConfig::new(WorkloadKind::Mixed, 1_000, 0.5, 42));

    c.bench_function("sequential mixed c50 1000", |b| {
        b.iter(|| run_sequential(&workload.initial_state, &workload.txs))
    });
    c.bench_function("access-list mixed c50 1000", |b| {
        b.iter(|| run_access_list(&workload.initial_state, &workload.txs, 4).unwrap())
    });
    c.bench_function("optimistic mixed c50 1000", |b| {
        b.iter(|| run_optimistic(&workload.initial_state, &workload.txs, 4).unwrap())
    });
}

criterion_group!(benches, executor_benchmarks);
criterion_main!(benches);
