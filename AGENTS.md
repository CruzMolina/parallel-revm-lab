# parallel-revm-lab Agent Notes

## Layout
- `crates/model`: deterministic account/storage state, transaction semantics, access sets, state hashing.
- `crates/workload`: seeded synthetic workload generation.
- `crates/executor`: sequential baseline, access-list waves, optimistic validation, reports, trace output.
- `crates/trace-model`: normalized block access traces and fixture parsing.
- `crates/analyzer`: conflict graph analysis, deterministic waves, dossier metrics, and report rendering.
- `crates/revm-smoke`: minimal revm bytecode smoke bridge into the analyzer and trace-pack schema.
- `crates/cli`: `parallel-revm-lab` binary.
- `docs`: design, correctness, benchmark, trace analysis, revm notes, failures, and engineering log.

## Commands
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.5 --threads 1,2 --seed-start 1 --seed-end 5`
- `cargo run -p parallel-revm-lab -- analyze-fixture --fixture fixtures/base-mini-trace.json --out reports/base-mini-trace.parallelism.json --markdown reports/base-mini-trace.md --html reports/base-mini-trace.html --trace reports/base-mini-trace.schedule.trace.json`
- `cargo run -p parallel-revm-lab -- analyze-trace --format geth-struct-logs --fixture fixtures/geth-mini-struct-logs.json --out reports/geth-mini.parallelism.json --markdown reports/geth-mini.md --html reports/geth-mini.html --trace reports/geth-mini.schedule.trace.json`
- `cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/demo-mini --workers 1,2,4,8 --out reports/demo-dossier.json --markdown reports/demo-dossier.md --html reports/demo-dossier.html --trace reports/demo-schedule.trace.json`
- `cargo test -p parallel-revm-lab-revm-smoke --all-features`

## Done Means
- Parallel executors produce the same final state hash as sequential execution.
- Trace analyzer reports are deterministic and honest about incomplete-read lower bounds.
- Trace-pack dossiers label provenance and keep gas-weighted scheduling theoretical.
- Public fixtures use valid EVM hex addresses, storage keys, and transaction hashes.
- All feature-gated code compiles with `--all-features`.
- Public docs match commands that were actually run.

## Determinism Rules
- Use `BTreeMap`, `BTreeSet`, sorted vectors, or explicit ordering for state, access sets, JSON, and hashes.
- Do not use `DefaultHasher` or nondeterministic map iteration for persistent outputs.
- Commit transaction effects in canonical input order.

## Benchmark Honesty
- Do not invent numbers, speedups, CI status, or revm support.
- Benchmark tables must come from commands in this repository.
- Describe throughput as synthetic scheduler/workbench throughput, not production TPS.

## Public Hygiene
- No secrets, API keys, env files, or private recruiting context.
- Keep broken experiments out of committed code.
- Review focus: deterministic output, sequential equivalence, edge cases, docs matching behavior.
