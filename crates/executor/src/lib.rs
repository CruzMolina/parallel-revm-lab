use std::collections::BTreeSet;
use std::time::Instant;

use parallel_revm_lab_model::{
    access_sets_conflict, execute_tx, ExecutionOutcome, ReadSet, State, StateHash, Tx, WriteSet,
};
use parallel_revm_lab_workload::{count_conflict_pairs, Workload};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    Sequential,
    AccessList,
    Optimistic,
    All,
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::Sequential => write!(f, "sequential"),
            ExecutionMode::AccessList => write!(f, "access-list"),
            ExecutionMode::Optimistic => write!(f, "optimistic"),
            ExecutionMode::All => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for ExecutionMode {
    type Err = ExecutorError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "sequential" => Ok(Self::Sequential),
            "access-list" | "access_list" => Ok(Self::AccessList),
            "optimistic" => Ok(Self::Optimistic),
            "all" => Ok(Self::All),
            other => Err(ExecutorError::UnknownMode(other.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("unknown execution mode `{0}`; expected sequential, access-list, optimistic, or all")]
    UnknownMode(String),
    #[error("rayon thread pool error: {0}")]
    ThreadPool(String),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionMetrics {
    pub elapsed_ns: u128,
    pub declared_conflict_pairs: u64,
    pub scheduler_deferrals: u64,
    pub validation_failures: u64,
    pub reexecuted_txs: u64,
    pub wave_count: u64,
    pub max_wave_width: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionResult {
    pub mode: ExecutionMode,
    pub final_state: State,
    pub state_hash: StateHash,
    pub metrics: ExecutionMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub version: String,
    pub workload: String,
    pub tx_count: usize,
    pub requested_conflict: f64,
    pub observed_conflict: f64,
    pub vm_steps: u64,
    pub seed: u64,
    pub threads: usize,
    pub rust_version: Option<String>,
    pub modes: Vec<ModeReport>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModeReport {
    pub mode: String,
    pub elapsed_ns: u128,
    pub tx_per_sec: f64,
    pub speedup_vs_sequential: Option<f64>,
    pub state_hash: String,
    pub declared_conflict_pairs: u64,
    pub scheduler_deferrals: u64,
    pub validation_failures: u64,
    pub reexecuted_txs: u64,
    pub reexecution_percent: f64,
    pub wave_count: u64,
    pub max_wave_width: usize,
    pub deterministic_passed: bool,
}

pub fn run_sequential(initial_state: &State, txs: &[Tx]) -> ExecutionResult {
    let started = Instant::now();
    let mut state = initial_state.clone();
    for tx in txs {
        let outcome = execute_tx(&state, tx);
        state.apply_delta(&outcome.delta);
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let state_hash = state.state_hash();
    ExecutionResult {
        mode: ExecutionMode::Sequential,
        final_state: state,
        state_hash,
        metrics: ExecutionMetrics {
            elapsed_ns,
            declared_conflict_pairs: count_conflict_pairs(txs),
            wave_count: 0,
            max_wave_width: 0,
            ..ExecutionMetrics::default()
        },
    }
}

pub fn run_access_list(
    initial_state: &State,
    txs: &[Tx],
    threads: usize,
) -> Result<ExecutionResult, ExecutorError> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .map_err(|err| ExecutorError::ThreadPool(err.to_string()))?;

    let started = Instant::now();
    let mut state = initial_state.clone();
    let mut pending: Vec<usize> = (0..txs.len()).collect();
    let mut wave_count = 0_u64;
    let mut max_wave_width = 0_usize;
    let mut scheduler_conflicts = 0_u64;

    while !pending.is_empty() {
        let mut wave = Vec::new();
        let mut next_pending = Vec::new();
        let mut wave_reads = BTreeSet::new();
        let mut wave_writes = BTreeSet::new();
        let mut blocked_reads = BTreeSet::new();
        let mut blocked_writes = BTreeSet::new();

        for tx_index in pending {
            let tx = &txs[tx_index];
            let conflicts_with_wave = access_sets_conflict(
                &wave_reads,
                &wave_writes,
                &tx.declared_reads,
                &tx.declared_writes,
            );
            let blocked_by_prior_deferred = access_sets_conflict(
                &blocked_reads,
                &blocked_writes,
                &tx.declared_reads,
                &tx.declared_writes,
            );
            if conflicts_with_wave || blocked_by_prior_deferred {
                scheduler_conflicts += 1;
                extend_sets(&mut blocked_reads, &mut blocked_writes, tx);
                next_pending.push(tx_index);
            } else {
                extend_sets(&mut wave_reads, &mut wave_writes, tx);
                wave.push(tx_index);
            }
        }

        let snapshot = state.clone();
        let outcomes: Vec<ExecutionOutcome> = pool.install(|| {
            wave.par_iter()
                .map(|tx_index| execute_tx(&snapshot, &txs[*tx_index]))
                .collect()
        });
        for outcome in outcomes {
            state.apply_delta(&outcome.delta);
        }

        wave_count += 1;
        max_wave_width = max_wave_width.max(wave.len());
        pending = next_pending;
    }

    let elapsed_ns = started.elapsed().as_nanos();
    let state_hash = state.state_hash();
    Ok(ExecutionResult {
        mode: ExecutionMode::AccessList,
        final_state: state,
        state_hash,
        metrics: ExecutionMetrics {
            elapsed_ns,
            declared_conflict_pairs: count_conflict_pairs(txs),
            scheduler_deferrals: scheduler_conflicts,
            wave_count,
            max_wave_width,
            ..ExecutionMetrics::default()
        },
    })
}

pub fn run_optimistic(
    initial_state: &State,
    txs: &[Tx],
    threads: usize,
) -> Result<ExecutionResult, ExecutorError> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .map_err(|err| ExecutorError::ThreadPool(err.to_string()))?;

    let started = Instant::now();
    let snapshot = initial_state.clone();
    let speculative: Vec<ExecutionOutcome> =
        pool.install(|| txs.par_iter().map(|tx| execute_tx(&snapshot, tx)).collect());
    let mut state = initial_state.clone();
    let mut reexecuted_txs = 0_u64;

    for (tx, outcome) in txs.iter().zip(speculative.iter()) {
        if outcome.reads_match(&state) {
            state.apply_delta(&outcome.delta);
        } else {
            let fresh = execute_tx(&state, tx);
            state.apply_delta(&fresh.delta);
            reexecuted_txs += 1;
        }
    }

    let elapsed_ns = started.elapsed().as_nanos();
    let state_hash = state.state_hash();
    Ok(ExecutionResult {
        mode: ExecutionMode::Optimistic,
        final_state: state,
        state_hash,
        metrics: ExecutionMetrics {
            elapsed_ns,
            declared_conflict_pairs: count_conflict_pairs(txs),
            validation_failures: reexecuted_txs,
            reexecuted_txs,
            wave_count: if txs.is_empty() { 0 } else { 1 },
            max_wave_width: txs.len(),
            ..ExecutionMetrics::default()
        },
    })
}

pub fn benchmark_report(
    workload: &Workload,
    mode: ExecutionMode,
    threads: usize,
    rust_version: Option<String>,
) -> Result<BenchmarkReport, ExecutorError> {
    let sequential = run_sequential(&workload.initial_state, &workload.txs);
    let sequential_elapsed = sequential.metrics.elapsed_ns.max(1);
    let mut reports = Vec::new();

    match mode {
        ExecutionMode::Sequential => {
            reports.push(mode_report(&sequential, workload.txs.len(), None, true));
        }
        ExecutionMode::AccessList => {
            let result = run_access_list(&workload.initial_state, &workload.txs, threads)?;
            let passed = result.final_state == sequential.final_state;
            reports.push(mode_report(
                &result,
                workload.txs.len(),
                Some(sequential_elapsed),
                passed,
            ));
        }
        ExecutionMode::Optimistic => {
            let result = run_optimistic(&workload.initial_state, &workload.txs, threads)?;
            let passed = result.final_state == sequential.final_state;
            reports.push(mode_report(
                &result,
                workload.txs.len(),
                Some(sequential_elapsed),
                passed,
            ));
        }
        ExecutionMode::All => {
            reports.push(mode_report(&sequential, workload.txs.len(), None, true));
            for result in [
                run_access_list(&workload.initial_state, &workload.txs, threads)?,
                run_optimistic(&workload.initial_state, &workload.txs, threads)?,
            ] {
                let passed = result.final_state == sequential.final_state;
                reports.push(mode_report(
                    &result,
                    workload.txs.len(),
                    Some(sequential_elapsed),
                    passed,
                ));
            }
        }
    }

    Ok(BenchmarkReport {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        workload: workload.config.kind.to_string(),
        tx_count: workload.txs.len(),
        requested_conflict: workload.config.requested_conflict,
        observed_conflict: workload.observed_conflict,
        vm_steps: workload.config.vm_steps,
        seed: workload.config.seed,
        threads: threads.max(1),
        rust_version,
        modes: reports,
    })
}

pub fn trace_json(report: &BenchmarkReport) -> serde_json::Value {
    let mut trace_events = Vec::new();
    let mut ts = 0_u128;
    for mode in &report.modes {
        let dur = (mode.elapsed_ns / 1_000).max(1);
        trace_events.push(serde_json::json!({
            "name": format!("mode:{}", mode.mode),
            "cat": "parallel-revm-lab",
            "ph": "X",
            "pid": 1,
            "tid": mode.mode,
            "ts": ts,
            "dur": dur,
            "args": {
                "state_hash": mode.state_hash,
                "deterministic_passed": mode.deterministic_passed,
                "declared_conflict_pairs": mode.declared_conflict_pairs,
                "scheduler_deferrals": mode.scheduler_deferrals,
                "validation_failures": mode.validation_failures,
                "reexecuted_txs": mode.reexecuted_txs,
                "wave_count": mode.wave_count
            }
        }));
        ts = ts.saturating_add(dur).saturating_add(10);
    }
    serde_json::json!({ "traceEvents": trace_events })
}

fn mode_report(
    result: &ExecutionResult,
    tx_count: usize,
    sequential_elapsed_ns: Option<u128>,
    deterministic_passed: bool,
) -> ModeReport {
    let elapsed_ns = result.metrics.elapsed_ns.max(1);
    let tx_per_sec = if tx_count == 0 {
        0.0
    } else {
        tx_count as f64 / (elapsed_ns as f64 / 1_000_000_000.0)
    };
    let speedup_vs_sequential = sequential_elapsed_ns.map(|baseline| {
        if elapsed_ns == 0 {
            0.0
        } else {
            baseline as f64 / elapsed_ns as f64
        }
    });
    let reexecution_percent = if tx_count == 0 {
        0.0
    } else {
        (result.metrics.reexecuted_txs as f64 / tx_count as f64) * 100.0
    };

    ModeReport {
        mode: result.mode.to_string(),
        elapsed_ns,
        tx_per_sec,
        speedup_vs_sequential,
        state_hash: result.state_hash.to_string(),
        declared_conflict_pairs: result.metrics.declared_conflict_pairs,
        scheduler_deferrals: result.metrics.scheduler_deferrals,
        validation_failures: result.metrics.validation_failures,
        reexecuted_txs: result.metrics.reexecuted_txs,
        reexecution_percent,
        wave_count: result.metrics.wave_count,
        max_wave_width: result.metrics.max_wave_width,
        deterministic_passed,
    }
}

fn extend_sets(reads: &mut ReadSet, writes: &mut WriteSet, tx: &Tx) {
    reads.extend(tx.declared_reads.iter().cloned());
    writes.extend(tx.declared_writes.iter().cloned());
}

#[cfg(test)]
mod tests {
    use parallel_revm_lab_workload::{generate_workload, WorkloadConfig, WorkloadKind};
    use proptest::prelude::*;

    use super::*;

    fn assert_parallel_equals_sequential(kind: WorkloadKind, txs: usize, conflict: f64, seed: u64) {
        let workload = generate_workload(WorkloadConfig::new(kind, txs, conflict, seed));
        let sequential = run_sequential(&workload.initial_state, &workload.txs);
        let access = run_access_list(&workload.initial_state, &workload.txs, 4).unwrap();
        let optimistic = run_optimistic(&workload.initial_state, &workload.txs, 4).unwrap();

        assert_eq!(sequential.state_hash, access.state_hash);
        assert_eq!(sequential.final_state, access.final_state);
        assert_eq!(sequential.state_hash, optimistic.state_hash);
        assert_eq!(sequential.final_state, optimistic.final_state);
    }

    #[test]
    fn access_list_and_optimistic_equal_sequential_on_fixed_workloads() {
        for kind in [
            WorkloadKind::Erc20,
            WorkloadKind::Storage,
            WorkloadKind::SwapLike,
            WorkloadKind::HotPool,
            WorkloadKind::Mixed,
        ] {
            assert_parallel_equals_sequential(kind, 120, 0.5, 42);
        }
    }

    #[test]
    fn high_contention_c95_does_not_panic_and_matches_sequential() {
        assert_parallel_equals_sequential(WorkloadKind::Mixed, 200, 0.95, 99);
    }

    #[test]
    fn zero_and_one_tx_edge_cases_match() {
        for txs in [0, 1] {
            assert_parallel_equals_sequential(WorkloadKind::Mixed, txs, 0.95, 1);
        }
    }

    #[test]
    fn benchmark_report_marks_parallel_modes_deterministic() {
        let workload = generate_workload(WorkloadConfig::new(WorkloadKind::Mixed, 80, 0.5, 123));
        let report = benchmark_report(&workload, ExecutionMode::All, 2, None).unwrap();
        assert_eq!(report.modes.len(), 3);
        assert!(report.modes.iter().all(|mode| mode.deterministic_passed));
        assert!(report
            .modes
            .iter()
            .all(|mode| mode.declared_conflict_pairs == workload.conflict_pairs));
        assert_eq!(report.modes[0].scheduler_deferrals, 0);
        assert!(report.modes[1].scheduler_deferrals > 0);
        assert_eq!(
            report.modes[2].validation_failures,
            report.modes[2].reexecuted_txs
        );
        assert!(serde_json::to_string_pretty(&report)
            .unwrap()
            .contains("declared_conflict_pairs"));
    }

    #[test]
    fn sequential_metrics_do_not_claim_wave_scheduling() {
        let workload = generate_workload(WorkloadConfig::new(WorkloadKind::Mixed, 10, 0.5, 7));
        let sequential = run_sequential(&workload.initial_state, &workload.txs);

        assert_eq!(sequential.metrics.wave_count, 0);
        assert_eq!(sequential.metrics.max_wave_width, 0);
    }

    #[test]
    fn vm_steps_add_cost_without_changing_state_semantics() {
        let cheap = generate_workload(WorkloadConfig::new(WorkloadKind::Storage, 40, 0.0, 9));
        let mut heavy_config = WorkloadConfig::new(WorkloadKind::Storage, 40, 0.0, 9);
        heavy_config.vm_steps = 32;
        let heavy = generate_workload(heavy_config);

        assert_eq!(cheap.initial_state, heavy.initial_state);
        assert_eq!(cheap.txs.len(), heavy.txs.len());
        assert!(heavy.txs.iter().all(|tx| tx.vm_steps == 32));

        let cheap_final = run_sequential(&cheap.initial_state, &cheap.txs);
        let heavy_final = run_sequential(&heavy.initial_state, &heavy.txs);
        assert_eq!(cheap_final.final_state, heavy_final.final_state);
        assert_eq!(cheap_final.state_hash, heavy_final.state_hash);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]

        #[test]
        fn random_small_workloads_match_sequential(
            seed in 0_u64..10_000,
            txs in 0_usize..80,
            conflict_step in 0_u8..=4,
            kind_step in 0_u8..=4,
        ) {
            let conflict = match conflict_step {
                0 => 0.0,
                1 => 0.2,
                2 => 0.5,
                3 => 0.7,
                _ => 0.95,
            };
            let kind = match kind_step {
                0 => WorkloadKind::Erc20,
                1 => WorkloadKind::Storage,
                2 => WorkloadKind::SwapLike,
                3 => WorkloadKind::HotPool,
                _ => WorkloadKind::Mixed,
            };
            let workload = generate_workload(WorkloadConfig::new(kind, txs, conflict, seed));
            let sequential = run_sequential(&workload.initial_state, &workload.txs);
            let access = run_access_list(&workload.initial_state, &workload.txs, 3).unwrap();
            let optimistic = run_optimistic(&workload.initial_state, &workload.txs, 3).unwrap();
            prop_assert_eq!(sequential.state_hash, access.state_hash);
            prop_assert_eq!(&sequential.final_state, &access.final_state);
            prop_assert_eq!(sequential.state_hash, optimistic.state_hash);
            prop_assert_eq!(&sequential.final_state, &optimistic.final_state);
        }
    }
}
