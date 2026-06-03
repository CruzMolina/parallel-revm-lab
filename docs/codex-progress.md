# Codex Progress

## 2026-06-03 Inspect

Plan:
- Inspect repository contents and local Rust tooling.
- Create the smallest public-grade workspace skeleton.
- Invariant improved: future work has deterministic rules, commands, and a progress log.
- Validation command: `rustc --version && cargo --version && git status --short`.

Result:
- Repository was empty except for `.git`.
- `rustc 1.92.0` and `cargo 1.92.0` are available.
- `just` is not installed in this environment.

## 2026-06-03 Workspace Skeleton

Plan:
- Add workspace manifests, agent instructions, docs directory, CI, and crate layout.
- Implement a first deterministic model/workload/executor/CLI vertical slice.
- Invariant improved: `cargo test --workspace --all-features` can prove sequential equivalence.
- Validation command: `cargo fmt --all -- --check`, then `cargo test --workspace --all-features`.

Result:
- `cargo fmt --all -- --check` initially failed with rustfmt-only diffs; `cargo fmt --all` fixed them.
- `cargo test --workspace --all-features` initially failed because the CLI referenced `serde` and `parallel-revm-lab-model` without declaring those dependencies.

## 2026-06-03 Scheduler Repair

Plan:
- Investigate access-list mismatches found by fixed tests and proptest.
- Preserve canonical transaction order when building waves.
- Invariant improved: no later conflicting transaction may skip an earlier deferred transaction.
- Validation command: `cargo test -p parallel-revm-lab-executor --lib`.

Result:
- Root cause found: later transactions could enter a wave even when they conflicted with earlier transactions deferred to the next wave.
- Added deferred-access barrier tracking.
- `cargo test -p parallel-revm-lab-executor --lib` passed: 5 tests.
- Kept the generated proptest regression seed for replay.

## 2026-06-03 Validation And Benchmark

Plan:
- Run required local validation commands.
- Generate the required release benchmark report and trace.
- Update public benchmark docs only from generated output.
- Invariant improved: docs and reports match actual commands.
- Validation commands: required Definition of Done commands plus `just --version`.

Result:
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: initially blocked because clippy was missing; `rustup component add clippy` installed it; rerun passed.
- `cargo run -p parallel-revm-lab -- --help`: passed.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `cargo run --release -p parallel-revm-lab -- bench --workload mixed --txs 1000 --conflict 0.5 --mode all --threads 4 --seed 42 --out reports/mixed-c50.json --trace reports/mixed-c50.trace.json`: passed.
- `just --version`: failed because `just` is not installed in this environment.

Benchmark snapshot:
- sequential: 1,540,832.05 synthetic tx/s, hash `ac90d19c91175700`.
- access-list: 16,767.11 synthetic tx/s, 0.011x vs sequential, 309 waves, hash matched.
- optimistic: 1,260,967.26 synthetic tx/s, 0.818x vs sequential, 731 re-executions, hash matched.

## 2026-06-03 Just Follow-Up

Plan:
- Re-run the two justfile gates after `just` became available.
- Update report-backed benchmark docs if `just bench-smoke` rewrites timing fields.
- Invariant improved: the local justfile recipes are verified, not only documented.
- Validation commands: `just validate`, `just bench-smoke`.

Result:
- `just --version`: passed with `just 1.51.0`.
- `just validate`: passed.
- `just bench-smoke`: passed and refreshed `reports/mixed-c50.json`.

Updated benchmark snapshot:
- sequential: 1,437,384.65 synthetic tx/s, hash `ac90d19c91175700`.
- access-list: 16,435.95 synthetic tx/s, 0.011x vs sequential, 309 waves, hash matched.
- optimistic: 1,206,029.67 synthetic tx/s, 0.839x vs sequential, 731 re-executions, hash matched.

## 2026-06-03 Skeptical Review

Plan:
- Inspect public docs and `crates/executor` from a skeptical protocol-engineering angle.
- Run the requested validation gates before changing behavior.
- Fix only material issues with minimal diffs.
- Invariant improved: benchmark metrics should not imply scheduler behavior for modes that do not schedule waves.
- Validation command: rerun the requested gates after the patch.

Result:
- Initial requested gates passed before changes.
- Found one material public-polish issue: sequential reports claimed `wave_count: 1` and `max_wave_width: tx_count`, which implied wave scheduling for the sequential baseline.
- Fixed sequential metrics to report zero wave fields and added `sequential_metrics_do_not_claim_wave_scheduling`.
- Clarified `docs/correctness.md` to mention the canonical-order deferred barrier in the access-list scheduler.
- Re-ran the requested gates after the patch; all passed.
- Refreshed `reports/mixed-c50.json` with the current release binary.

Updated benchmark snapshot:
- sequential: 1,413,010.15 synthetic tx/s, hash `ac90d19c91175700`, zero wave metrics.
- access-list: 16,937.54 synthetic tx/s, 0.012x vs sequential, 309 waves, hash matched.
- optimistic: 1,375,042.97 synthetic tx/s, 0.973x vs sequential, 731 re-executions, hash matched.

## 2026-06-03 Release Repair

Plan:
- Fix correctness-test/docs mismatch by making property tests compare full final `State` values.
- Split ambiguous report metrics into declared conflicts, scheduler deferrals, validation failures, re-executions, and wave shape.
- Add `--vm-steps` for deterministic synthetic interpreter work and document both cheap and heavier benchmark cases from real reports.
- Keep `revm` integration notes honest and avoid broken feature-gated code.
- Validation command: full required release gate after reports/docs are refreshed.

Result:
- Property tests now compare both final state hashes and full final `State` values.
- Reports now include `declared_conflict_pairs`, `scheduler_deferrals`, `validation_failures`, `reexecuted_txs`, `wave_count`, and `max_wave_width`.
- Added `--vm-steps` to `bench`, `verify`, and `inspect`; the synthetic CPU loop does not affect state semantics.
- Generated `reports/mixed-c50.json` for cheap mixed transactions and `reports/storage-c0-vmsteps.json` for heavier low-contention storage transactions.
- Did not add `revm` code; `docs/revm-integration-notes.md` explains that no clean smoke adapter was included.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`: passed, 300 workload/thread combinations.
- `cargo run --release -p parallel-revm-lab -- bench --workload mixed --txs 1000 --conflict 0.5 --mode all --threads 4 --seed 42 --out reports/mixed-c50.json --trace reports/mixed-c50.trace.json`: passed.

Current benchmark snapshot:
- cheap mixed c50: access-list 0.013x and optimistic 0.840x vs sequential; overhead dominates.
- heavier storage c0 with `vm_steps=50000`: access-list 3.232x and optimistic 2.154x vs sequential; parallelism wins under low contention.
