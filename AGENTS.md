# parallel-revm-lab Agent Notes

## Layout
- `crates/model`: deterministic account/storage state, transaction semantics, access sets, state hashing.
- `crates/workload`: seeded synthetic workload generation.
- `crates/executor`: sequential baseline, access-list waves, optimistic validation, reports, trace output.
- `crates/cli`: `parallel-revm-lab` binary.
- `docs`: design, correctness, benchmark, failure, and progress notes.

## Commands
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.5 --threads 1,2 --seed-start 1 --seed-end 5`

## Done Means
- Parallel executors produce the same final state hash as sequential execution.
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
