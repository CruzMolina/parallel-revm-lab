# Engineering Log

This log records validation history, failures, benchmark commands, and engineering decisions. It intentionally avoids process-specific branding so the public repo reads as a normal Rust systems artifact.

## 2026-06-03 Initial State

- Current repo started as a deterministic synthetic scheduler lab.
- Current local toolchain: stable Rust, `rustc 1.92.0` observed in benchmark reports.
- Existing validation gates: fmt, clippy, full workspace tests, and synthetic `verify`.
- Existing public issue: the repo did not yet analyze fixture or real EVM access traces.

## 2026-06-03 Upgrade Plan

Milestones:

1. Add deterministic trace model and fixture parser.
2. Add conflict graph, deterministic waves, hot-state rankings, and report renderers.
3. Add `analyze-fixture` and best-effort `analyze-block` CLI surfaces.
4. Add a minimal revm smoke crate that executes bytecode and feeds observations into the analyzer.
5. Update README/docs/CI/justfile and regenerate committed reports.
6. Run full validation and record exact results.

Assumptions:

- Fixture mode is the reliable CI path.
- Live RPC trace APIs vary by provider, so `analyze-block` must never be required for CI.
- `revm 40.0.3` requires Rust 1.91; the workspace MSRV claim is updated accordingly.

## 2026-06-03 Failures And Repairs

### Missing CLI Dependencies

- Symptom: `cargo test --workspace --all-features` failed with unresolved `serde` and `parallel_revm_lab_model` in `crates/cli/src/main.rs`.
- Root cause: the CLI wrote generic JSON and formatted `AccessKey` values but did not declare direct dependencies on the crates that define those symbols.
- Fix: added `serde` and `parallel-revm-lab-model` to `crates/cli/Cargo.toml`.
- Coverage: existing CLI smoke tests cover binary compilation.

### Access-List Reordered Deferred Conflicts

- Symptom: executor tests found access-list final state hashes diverging from sequential hashes on mixed and high-contention workloads.
- Root cause: the wave builder admitted later transactions that were independent of the current wave but conflicted with earlier transactions already deferred to the next wave.
- Fix: track access sets for earlier deferred transactions and prevent later conflicting transactions from joining the current wave.
- Coverage: fixed examples, high-contention tests, and proptest sequential-equivalence checks.

### Correctness Docs Outran Property Tests

- Symptom: `docs/correctness.md` said property tests compared full final `State` values, but randomized assertions only compared hashes.
- Root cause: fixed examples compared full state while the property test stopped at the hash invariant.
- Fix: property tests now compare both final state hash and full final `State`.
- Coverage: strengthened existing `random_small_workloads_match_sequential`.

### Ambiguous Conflict Metric

- Symptom: benchmark reports used one conflict-like field for scheduler deferrals, declared conflicts, and optimistic validation failures.
- Root cause: distinct metrics were collapsed into one public field.
- Fix: report schema now splits declared conflict pairs, scheduler deferrals, validation failures, re-executed transactions, wave count, and max wave width.
- Coverage: benchmark report tests assert split metric consistency.

### Sequential Baseline Claimed Wave Metrics

- Symptom: sequential reports claimed a wave count and width even though no wave scheduler ran.
- Root cause: sequential metrics reused a wave-shaped summary field.
- Fix: sequential metrics now report zero wave fields.
- Coverage: `sequential_metrics_do_not_claim_wave_scheduling`.

### Nonce Writes Could Wrap Negative Values

- Symptom: writing a negative nonce through the model's generic state writer cast through `as u64`, which can wrap.
- Root cause: unchecked numeric cast in `State::write` for `AccountNonce`.
- Fix: clamp negative nonce writes to zero and saturate too-large values to `u64::MAX`.
- Coverage: `nonce_write_saturates_instead_of_wrapping`.

### Trace Reads Can Be Incomplete

- Symptom: incomplete read traces were initially described as conservative conflict analysis.
- Root cause: missing reads make conflict counts lower bounds, not conservative over-approximations.
- Fix: analyzer marks such reports with `declared-read-write-lower-bound` and emits explicit lower-bound warnings.
- Coverage: committed fixture test requires incomplete-read warnings.

## 2026-06-03 revm Smoke Decision

- Attempt: inspected `cargo info revm` and local crate sources, then used a compile-tested minimal API: `Context::mainnet`, `CacheDB<EmptyDB>`, `TxEnv::builder`, and `build_mainnet().transact(...)`.
- Version: `revm = 40.0.3`, exact-pinned, `default-features = false`, `features = ["std"]`.
- MSRV impact: `revm 40.0.3` declares `rust-version = 1.91.0`; workspace `rust-version` now matches that floor.
- Implementation: `crates/revm-smoke` executes tiny bytecode fixtures against in-memory revm state and converts known fixture behavior into this repo's normalized trace model.
- Limitation: this is not a general EVM tracer or block replay adapter. It is a compiling bridge proving the analyzer can consume observations from real EVM bytecode execution.

## 2026-06-03 Live RPC Decision

- Attempt scope: CLI surface, environment handling, and provider caveat review.
- Decision: keep `analyze-block` as a clear, non-secret-printing failure path until a provider-specific trace normalizer is implemented with fixtures.
- Blocker: Ethereum JSON-RPC trace/debug APIs differ across providers and clients, and reliable read/write extraction cannot be claimed without endpoint-specific fixtures.
- Reliable path for this revision: `analyze-fixture` and committed normalized fixtures.

## 2026-06-03 Validation Results

Commands run after the trace analyzer and revm smoke upgrade:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `cargo run -p parallel-revm-lab -- analyze-fixture --fixture fixtures/base-mini-trace.json --out reports/base-mini-trace.parallelism.json --markdown reports/base-mini-trace.md --html reports/base-mini-trace.html --trace reports/base-mini-trace.schedule.trace.json`: passed, 12 txs, 7 conflicts, 3 waves, max width 7, hash `45027250c9c34541`.
- `cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json`: passed; access-list 3.215x and optimistic 2.295x vs sequential.
- `cargo test -p parallel-revm-lab-revm-smoke --all-features`: passed, 3 tests.
- `just --version`: passed with `just 1.51.0`.
- `just validate`: passed.
- `just analyze-fixture`: passed.
- `just bench-smoke`: passed; refreshed `reports/mixed-c50.json`.

Current fixture report:

- `tx_count`: 12
- `conflict_pair_count`: 7
- `conflict_percentage`: 10.606%
- `wave_count`: 3
- `max_wave_width`: 7
- `critical_path_length`: 3
- `theoretical_parallelism_ceiling`: 4.000x

Current benchmark snapshots:

- cheap mixed c50: access-list 0.012x and optimistic 0.967x vs sequential; scheduling overhead dominates access-list for this tiny synthetic batch.
- heavier storage c0 with `vm_steps=50000`: access-list 3.215x and optimistic 2.295x vs sequential; parallelism wins under low contention.
