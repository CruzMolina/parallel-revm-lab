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
