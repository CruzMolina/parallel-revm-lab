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
- Initial implementation: `crates/revm-smoke` executed tiny bytecode fixtures against in-memory revm state and converted known fixture behavior into this repo's normalized trace model.
- Superseding implementation: the final credibility pass below replaced fixture-derived storage observations with an actual `revm` inspector that records `SLOAD` and `SSTORE` during bytecode execution.
- Limitation: this is not a general EVM tracer or block replay adapter. It is a compiling bridge proving the analyzer can consume storage observations from real EVM bytecode execution.

## 2026-06-03 Live RPC Decision

- Attempt scope: CLI surface, environment handling, and provider caveat review.
- Decision: keep `analyze-block` as a clear, non-secret-printing failure path until a provider-specific trace normalizer is implemented with fixtures.
- Blocker: Ethereum JSON-RPC trace/debug APIs differ across providers and clients, and reliable read/write extraction cannot be claimed without endpoint-specific fixtures.
- Reliable paths for this revision: `analyze-trace`, `analyze-fixture`, and committed offline fixtures.

## 2026-06-03 Validation Results

Commands run after the trace analyzer and revm smoke upgrade:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `cargo run -p parallel-revm-lab -- analyze-fixture --fixture fixtures/base-mini-trace.json --out reports/base-mini-trace.parallelism.json --markdown reports/base-mini-trace.md --html reports/base-mini-trace.html --trace reports/base-mini-trace.schedule.trace.json`: passed, 12 txs, 7 conflicts, 3 waves, max width 7, hash `551135a34ecf7b50`.
- `cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json`: passed; access-list 3.190x and optimistic 2.300x vs sequential.
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

- cheap mixed c50: access-list 0.011x and optimistic 0.806x vs sequential; scheduling overhead dominates access-list for this tiny synthetic batch.
- heavier storage c0 with `vm_steps=50000`: access-list 3.190x and optimistic 2.300x vs sequential; parallelism wins under low contention.

## 2026-06-03 Final Credibility Pass Plan

Milestones:

1. Replace synthetic mnemonic fixture addresses with valid `0x`-prefixed 20-byte hex addresses and keep the same 12-tx/3-wave shape.
2. Validate fixture addresses, tx hashes, and storage keys with clear `tx_index` context.
3. Add a working offline `analyze-trace --format geth-struct-logs` path for a tiny committed struct-log fixture.
4. Replace revm-smoke's fixture-derived access observations with an actual revm inspector that records `SLOAD` and `SSTORE`.
5. Demote live `analyze-block` from the README headline path and make offline trace/fixture analysis the working public story.
6. Regenerate reports and run the full validation gate.

Assumptions:

- No live RPC support is added in this pass.
- Geth struct-log support targets the tiny documented fixture shape only; wider provider support remains future work.
- Incomplete or stack-limited traces must emit lower-bound warnings instead of pretending complete coverage.

## 2026-06-03 Final Credibility Pass Implementation

- Replaced mnemonic fixture addresses with valid `0x` plus 40-hex EVM addresses while preserving the 12-transaction, 7-conflict, 3-wave fixture shape.
- Added trace validation for tx hashes, sender/recipient addresses, access addresses, and storage keys with `tx_index` and access-position context.
- Added `analyze-trace --format geth-struct-logs` for the committed tiny sanitized struct-log fixture. It derives storage slots from `SLOAD` and `SSTORE` stack values and marks reads incomplete outside those observations.
- Replaced revm-smoke's fixture-derived storage observations with a real `revm` inspector over `build_mainnet_with_inspector(...).inspect_tx(...)`.
- Hid `analyze-block` from the public help path and documented it as future/provider-specific RPC work.
- Regenerated normalized fixture and Geth struct-log reports.

## 2026-06-03 Final Credibility Pass Validation Results

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed, 38 tests.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `cargo run -p parallel-revm-lab -- analyze-fixture --fixture fixtures/base-mini-trace.json --out reports/base-mini-trace.parallelism.json --markdown reports/base-mini-trace.md --html reports/base-mini-trace.html --trace reports/base-mini-trace.schedule.trace.json`: passed, 12 txs, 7 conflicts, 3 waves, max width 7, hash `551135a34ecf7b50`.
- `cargo run -p parallel-revm-lab -- analyze-trace --format geth-struct-logs --fixture fixtures/geth-mini-struct-logs.json --out reports/geth-mini.parallelism.json --markdown reports/geth-mini.md --html reports/geth-mini.html --trace reports/geth-mini.schedule.trace.json`: passed, 3 txs, 1 conflict, 2 waves, max width 2, hash `1c9e9d1d244efdde`.
- `cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json`: passed; access-list 3.190x and optimistic 2.300x vs sequential.
- `cargo test -p parallel-revm-lab-revm-smoke --all-features`: passed, 3 tests.
- `just validate`: passed.
- `just analyze-fixture`: passed, hash `551135a34ecf7b50`.
- `just bench-smoke`: passed; mixed c50 access-list 0.011x and optimistic 0.806x vs sequential.

## 2026-06-03 Contention Dossier Pass Plan

Milestones:

1. Add a compact trace-pack schema under `trace-packs/<name>/manifest.json` plus per-block JSON files, with deterministic normalization and validation. Validation command: `cargo test -p parallel-revm-lab-trace-model --all-features`.
2. Add range-level dossier analysis over trace packs: gas-weighted critical path, conflict concentration, hot contracts/slots, and deterministic worker simulation. Validation command: `cargo test -p parallel-revm-lab-analyzer --all-features`.
3. Wire `analyze-trace-pack` and `recommend-access-lists` into the CLI and generate a committed demo dossier. Validation command: `cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/demo-mini --workers 1,2,4,8 --out reports/demo-dossier.json --markdown reports/demo-dossier.md --html reports/demo-dossier.html --trace reports/demo-schedule.trace.json`.
4. Add optional `collect-block-range` tooling and a compact Geth JS storage tracer without making live RPC part of CI. Validation command: `cargo test -p parallel-revm-lab --all-features collect`.
5. Add a revm-smoke trace-pack bridge proving `revm` bytecode observations can validate and analyze through the new schema. Validation command: `cargo test -p parallel-revm-lab-revm-smoke --all-features`.
6. Polish README, case-study docs, just recipes, CI, and generated artifacts while keeping Base-range reports as reproduction instructions unless real RPC data is collected. Validation command: full fmt, clippy, workspace tests, verify sweep, dossier generation, and release storage bench.

Assumptions:

- No debug-capable Base RPC URL is available by default; committed Base-range materials must remain reproduction instructions, not claimed real-chain results.
- Demo trace packs are intentionally tiny and sanitized.
- Gas-weighted scheduling is a theoretical deterministic list-scheduling model, not measured throughput.
- Live RPC errors must redact URLs and tokens.

## 2026-06-03 Contention Dossier Pass Implementation

- Added `trace-packs/<name>/manifest.json` plus per-block JSON schema with deterministic normalization, valid EVM hex checks, duplicate access deduplication, and gas-optional validation.
- Added `trace-packs/demo-mini`, a tiny synthetic/demo trace pack over synthetic blocks `900000001` and `900000002`; it is explicitly not real Base data.
- Added trace-pack dossier analysis with conflict graph, deterministic waves, gas-weighted conflict percentage, gas critical path, worker simulation, hot contracts, hot slots, contention concentration, and CSV sidecars.
- Added `analyze-trace-pack` and `recommend-access-lists` CLI commands. Recommendations are labeled observed access hints, not complete production access lists.
- Added `collect-block-range` with optional `debug_traceTransaction`/Geth JS tracer collection, `--dry-run`, `--resume`, `--max-transactions`, receipt gas/status capture, and RPC URL redaction.
- Added `tracers/geth-storage-access-tracer.js` and `docs/geth-tracer.md`.
- Added revm-smoke `smoke_trace_pack` plus `cargo run -p parallel-revm-lab-revm-smoke --example emit_trace_pack` to prove `revm` bytecode -> inspector observations -> trace pack -> dossier.
- Added `case-studies/demo-trace-pack/` generated artifacts and `case-studies/base-38014901-38014910/README.md` reproduction instructions only. No fake Base report was committed.

Current demo dossier:

- `tx_count`: 7
- `conflict_pair_count`: 2
- `conflict_percentage`: 22.222%
- `critical_path_length_by_tx`: 4
- `gas_weighted_critical_path`: 405
- `theoretical_parallelism_ceiling_by_tx`: 1.750x
- `theoretical_parallelism_ceiling_by_gas`: 1.593x

## 2026-06-03 Contention Dossier Pass Validation Results

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed, 53 tests.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/demo-mini --workers 1,2,4,8 --out reports/demo-dossier.json --markdown reports/demo-dossier.md --html reports/demo-dossier.html --trace reports/demo-schedule.trace.json`: passed, 2 blocks, 7 txs, 2 conflicts, gas ceiling 1.593x.
- `cargo run -p parallel-revm-lab -- recommend-access-lists --trace-dir trace-packs/demo-mini --out reports/access-list-recommendations.json`: passed, 7 observed access hints.
- `cargo run -p parallel-revm-lab-revm-smoke --example emit_trace_pack`: passed; refreshed `trace-packs/revm-smoke-mini` and `reports/revm-smoke-dossier.*`.
- `cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json`: passed; access-list 3.190x and optimistic 2.300x vs sequential.
- `just validate-full`: passed; reran fmt, clippy, workspace tests, verifier sweep, fixture analysis, trace-pack dossier generation, revm smoke tests, revm trace-pack emission, and the release storage benchmark smoke.

Note: one earlier focused multi-crate test attempt hit a transient Apple clang linker segmentation fault while linking `parallel-revm-lab-revm-smoke`; rerunning the revm smoke test and the full workspace test passed.

## 2026-06-03 Review Finding Fix Plan

Milestones:

1. Reject complete-gas trace-pack blocks whose `total_gas_used` does not equal the sum of transaction `gas_used` values.
2. Reject adjacent block hash discontinuity when both previous `block_hash` and current `parent_hash` are present.
3. Preserve tx-level trace warnings in block and range dossiers, including incomplete-read/provider warnings.
4. Make `collect-block-range --resume` durable at block granularity by writing validated block files immediately.
5. Make collector dry-run iterate every requested block and update docs accordingly.

## 2026-06-03 Review Finding Fix Implementation

- Added trace-pack validation for complete-gas reconciliation and parent-hash continuity.
- Added analyzer warning collection from normalized block and transaction trace warnings.
- Added collector helpers for all-block dry-run iteration, immediate block persistence, and resumed-block validation.
- Updated public docs and failure notes so dry-run/resume/gas-weighted semantics match behavior.

## 2026-06-03 Review Finding Fix Validation Results

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed, 58 tests.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `just validate-full`: passed; reran fmt, clippy, workspace tests, verifier sweep, fixture analysis, trace-pack dossier generation, revm smoke tests, revm trace-pack emission, and the release storage benchmark smoke.
- Review-fix artifacts: demo dossier now reports 9 incomplete-trace warning signals; revm-smoke dossier now reports 7 incomplete-trace warning signals.
- Release storage smoke: access-list 3.190x and optimistic 2.300x vs sequential, all modes state hash `1940c0cfba64e3cb`.

## 2026-06-03 Final Base Dossier Pass Plan

Milestones:

1. Check real RPC capability first without printing secrets. If `BASE_RPC_URL` is missing or unsupported, document the exact blocker and keep the offline path perfect.
2. Clean demo provenance so committed synthetic trace packs cannot be confused with real Base data: synthetic chain names and non-Base block numbers.
3. Add missing report semantics needed for a real sample: tx coverage percentage, overlapping transaction percentage, range wave/max-width fields, and worst serializing transaction ranking.
4. Add `rpc-capability-check` and optional Markdown rendering for observed access hints. Keep live RPC out of CI.
5. If a debug-capable Base RPC endpoint is available, collect the smallest compact real sample first: Base block `38014901`, up to 25 transactions, custom Geth JS storage tracer, then generate dossier, access hints, provenance, and optimization memo.
6. If the sample succeeds and output remains small, attempt the full `38014901-38014910` range. If it does not, keep the small sample as the real artifact.
7. If real collection is blocked, update README and `case-studies/base-38014901-38014910/` with exact reproduction instructions and the observed local blocker, but do not create fake real-chain findings.
8. Regenerate committed demo artifacts, run full validation, review for misleading claims and secrets, and commit.

Current environment note:

- `BASE_RPC_URL` was not set during the initial capability inspection, so this run cannot collect real Base data unless the environment changes before the collection milestone.

## 2026-06-03 Final Base Dossier Pass Implementation Notes

- Cleaned `trace-packs/demo-mini` provenance to `synthetic-base-shaped` and moved demo block numbers to `900000001-900000002`.
- Cleaned the normalized `fixtures/base-mini-trace.json` fixture to `synthetic-base-shaped-fixture` block `900000000` and regenerated `reports/base-mini-trace.*`.
- Added `source_tx_count` to trace-pack blocks so partial real samples can report transaction coverage honestly.
- Added range-level `overlapping_tx_percentage`, wave/max-width totals, worst serializing transaction ranking, and Markdown rendering for observed access hints.
- Added `rpc-capability-check` to verify block availability, receipts, debug tracing, custom JavaScript tracer support, and struct-log fallback support without printing RPC URLs.
- Regenerated demo dossier and access-hint artifacts from the renamed synthetic trace pack.

RPC capability result in this environment:

- `cargo run -p parallel-revm-lab -- rpc-capability-check --chain base --block 38014901`: failed before network access because no `BASE_RPC_URL` or `ETH_RPC_URL` was set. No RPC URL was printed.

## 2026-06-03 Final Base Dossier Pass Validation Results

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed, 65 tests plus doctests.
- `cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/demo-mini --workers 1,2,4,8 --out reports/demo-dossier.json --markdown reports/demo-dossier.md --html reports/demo-dossier.html --trace reports/demo-schedule.trace.json`: passed, 2 synthetic blocks, 7 txs, 2 conflicts, 57.143% overlapping txs, gas ceiling 1.593x.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json`: passed; access-list 3.190x and optimistic 2.300x vs sequential, all modes state hash `1940c0cfba64e3cb`.
- `cargo run -p parallel-revm-lab -- rpc-capability-check --chain base --block 38014901`: failed as expected in this environment because no `BASE_RPC_URL` or `ETH_RPC_URL` was set; no URL or token was printed.
- `cargo run -p parallel-revm-lab-revm-smoke --example emit_trace_pack`: passed; refreshed `trace-packs/revm-smoke-mini` and `reports/revm-smoke-dossier.*`.

Real collection status: blocked by missing `BASE_RPC_URL`; no real Base trace pack, dossier, access hints, or optimization memo were produced.
