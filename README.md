# parallel-revm-lab

Deterministic parallel execution and EVM contention analysis in Rust.

This repository is a protocol-engineering lab for studying deterministic parallel execution, EVM-shaped state contention, offline trace packs, hot-state bottlenecks, and theoretical scheduling ceilings. It is not a production execution client, does not replay full blocks, and does not claim production TPS or Ggas/s.

## Working Commands

Verify the synthetic scheduler invariant:

```sh
cargo run -p parallel-revm-lab -- verify \
  --workload mixed \
  --txs 100 \
  --conflicts 0.0,0.2,0.5,0.7,0.95 \
  --threads 1,2,4 \
  --seed-start 1 \
  --seed-end 20
```

Analyze the committed demo trace pack:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/demo-mini \
  --workers 1,2,4,8 \
  --out reports/demo-dossier.json \
  --markdown reports/demo-dossier.md \
  --html reports/demo-dossier.html \
  --trace reports/demo-schedule.trace.json
```

Run the heavy low-contention synthetic benchmark snapshot:

```sh
cargo run --release -p parallel-revm-lab -- bench \
  --workload storage \
  --txs 1000 \
  --conflict 0.0 \
  --mode all \
  --threads 4 \
  --seed 42 \
  --vm-steps 50000 \
  --out reports/storage-c0-vmsteps.json
```

## Killer Case Study

The dossier path targets a public Base-style block range:

- `case-studies/base-38014901-38014910/`
- start block `38014901`
- end block `38014910`

No real Base report is committed for that range. The directory contains reproduction instructions only, because real collection requires a user-supplied Base RPC endpoint that supports `debug_traceTransaction` with custom Geth JavaScript tracers.

The committed proof path is:

- `trace-packs/demo-mini/`: tiny synthetic/demo trace pack, not real Base data.
- `case-studies/demo-trace-pack/summary.md`: polished dossier summary.
- `case-studies/demo-trace-pack/dossier.html`: static report.

Current demo dossier:

| metric | value |
| --- | ---: |
| blocks | 2 |
| transactions | 7 |
| conflict pairs | 2 |
| conflict percentage | 22.222% |
| critical path by tx | 4 |
| gas-weighted critical path | 405 |
| theoretical ceiling by tx | 1.750x |
| theoretical ceiling by gas | 1.593x |

Worker simulation reaches the gas critical-path bound at 2 workers in the demo. More workers do not help because observed dependencies are already binding.

## Optional Real-Chain Collection

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014910 \
  --rpc-url "$BASE_RPC_URL" \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-38014910 \
  --resume
```

Use `--dry-run` first to check block availability. RPC URLs are never printed, and live RPC collection is not required in CI.

Then analyze the collected trace pack:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-38014910 \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-38014910/dossier.json \
  --markdown case-studies/base-38014901-38014910/summary.md \
  --html case-studies/base-38014901-38014910/dossier.html \
  --trace case-studies/base-38014901-38014910/schedule.trace.json
```

## What Is Real Vs Synthetic

| Layer | Provenance | What it proves | What it does not prove |
| --- | --- | --- | --- |
| synthetic scheduler | generated workload | sequential equivalence of implemented schedulers | EVM semantics |
| demo trace pack | synthetic/demo fixture | dossier schema, gas-weighted scheduling, hot-state analysis | Base mainnet behavior |
| Geth tracer | optional RPC collection tool | compact storage observation path where providers support JS tracers | provider-wide trace compatibility |
| revm smoke | real `revm` bytecode | inspector storage observations can feed trace packs and dossiers | full block replay |

## What It Demonstrates

- Deterministic state, access-key, delta, and stable hash model.
- Sequential baseline, access-list wave scheduler, and optimistic validation executor.
- Compact trace-pack schema with provenance, block files, gas, and validated EVM hex fields.
- Dossier analysis: conflict graph, waves, gas-weighted critical path, worker simulation, hot contracts, hot slots, concentration, and warnings.
- Optional Geth JS storage tracer and `collect-block-range` command for user-collected RPC trace packs.
- Observed access-hint recommendations that explicitly are not production-ready Ethereum access lists.
- A `revm 40.0.3` smoke path that emits a trace pack from inspector-captured `SLOAD`/`SSTORE`.

## Validation

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

If `just` is installed:

```sh
just validate
```

## Limitations

- Trace quality determines analyzer quality; incomplete reads can hide conflicts.
- The Geth tracer records storage opcodes only.
- `collect-block-range` depends on provider support for debug tracing and JavaScript tracers.
- Worker simulation is theoretical deterministic scheduling, not measured throughput.
- Observed access hints are not complete production access lists.
- Stable hashes are FNV-1a report hashes, not cryptographic commitments.

## Repository Map

- `crates/model`: deterministic state, transactions, deltas, conflict detection, stable hashing.
- `crates/workload`: seeded synthetic workload generation.
- `crates/executor`: sequential, access-list, optimistic execution, reports, traces.
- `crates/trace-model`: normalized traces and trace-pack schema/validation.
- `crates/analyzer`: conflict graph, dossier metrics, report renderers, observed access hints.
- `crates/revm-smoke`: minimal revm bytecode smoke bridge with trace-pack emitter.
- `crates/cli`: `parallel-revm-lab` command-line interface.
- `tracers/geth-storage-access-tracer.js`: compact Geth JS storage tracer.
- `trace-packs/demo-mini`: committed synthetic demo trace pack.
- `case-studies/demo-trace-pack`: committed demo dossier.
- `case-studies/base-38014901-38014910`: real-collection reproduction instructions.

## What To Review In 90 Seconds

1. `case-studies/demo-trace-pack/summary.md`
2. `case-studies/demo-trace-pack/dossier.html`
3. `crates/trace-model/src/trace_pack.rs`
4. `crates/analyzer/src/dossier.rs`
5. `crates/cli/src/main.rs`
6. `tracers/geth-storage-access-tracer.js`
7. `docs/geth-tracer.md`
