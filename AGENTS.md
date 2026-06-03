# parallel-revm-lab Agent Notes

## Layout
- `crates/model`: deterministic account/storage state, transaction semantics, access sets, state hashing.
- `crates/workload`: seeded synthetic workload generation.
- `crates/executor`: sequential baseline, access-list waves, optimistic validation, reports, trace output.
- `crates/trace-model`: normalized block access traces and fixture parsing.
- `crates/analyzer`: conflict graph analysis, deterministic waves, and report rendering.
- `crates/revm-smoke`: minimal revm bytecode smoke bridge into the analyzer.
- `crates/cli`: `parallel-revm-lab` binary.
- `docs`: design, correctness, benchmark, trace analysis, revm notes, failures, and engineering log.

## Commands
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.5 --threads 1,2 --seed-start 1 --seed-end 5`
- `cargo run -p parallel-revm-lab -- analyze-fixture --fixture fixtures/base-mini-trace.json --out reports/base-mini-trace.parallelism.json --markdown reports/base-mini-trace.md --html reports/base-mini-trace.html --trace reports/base-mini-trace.schedule.trace.json`
- `cargo test -p parallel-revm-lab-revm-smoke --all-features`

## Done Means
- Parallel executors produce the same final state hash as sequential execution.
- Trace analyzer reports are deterministic and honest about incomplete-read lower bounds.
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
