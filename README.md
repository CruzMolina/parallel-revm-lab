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

## Real Base Case Study

Real Base trace-backed case study available:

- `case-studies/base-38014901-execution-dossier/executive-summary.md`
- `case-studies/base-38014901-execution-dossier/dossier.html`
- `case-studies/base-38014901-execution-dossier/optimization-memo.md`
- `trace-packs/base-38014901-full/`

This is a full-block real trace-backed analysis of Base block `38014901`.

Headline findings:

| metric | value |
| --- | ---: |
| block | `38014901` |
| txs covered | 436 of 436 (100.000%) |
| gas covered | 71,655,982 |
| observed accesses | 27,034 |
| unique contracts | 386 |
| unique storage slots | 4,321 |
| conflict pairs | 5,052 (5.327%) |
| gas-weighted conflict percentage | 7.172% |
| overlapping transactions | 403 (92.431%) |
| waves | 60 |
| max wave width | 106 |
| gas-weighted critical path | 16,951,990 |
| theoretical ceiling by gas | 4.227x |
| canonical 16-worker simulated speedup | 4.227x |
| critical-path scheduler speedup at 4 workers | 4.000x |
| top hot contract | `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913` |
| top conflict slot | `0x4200000000000000000000000000000000000006:0x0d52ad225b9f8da090dc37c741705dabc30f648dce00d7b0cab66994a1261ea6` |

The block has high observed key overlap but substantially less write-dependent serialization: 92.431% of transactions shared at least one observed key, while pairwise conflicts were 5.327%. Scheduler ablation shows critical-path ready-queue priority improving the canonical schedule by 7.460% at 4 workers and reaching the gas critical-path bound at 8 and 16 workers.

Caveat: the tracer records observed storage accesses, not complete EVM access lists, and the schedule is theoretical. The trace-derived benchmark is synthetic execution over observed topology, not full block replay.

The larger public target range remains:

- `case-studies/base-38014901-38014910/`
- start block `38014901`
- end block `38014910`

No full-range real Base report is committed for that range. A dry run found 3,676 transactions across those ten blocks, so full tracing was deferred rather than committing a large, slow, partial artifact.

## Offline Demo Dossier

The committed proof path is:

- `trace-packs/demo-mini/`: tiny synthetic/demo trace pack using `synthetic-base-shaped` and synthetic block numbers, not real Base data.
- `case-studies/demo-trace-pack/summary.md`: polished dossier summary.
- `case-studies/demo-trace-pack/dossier.html`: static report.

Current demo dossier:

| metric | value |
| --- | ---: |
| range | `synthetic-base-shaped` blocks `900000001-900000002` |
| blocks | 2 |
| transactions | 7 |
| conflict pairs | 2 |
| conflict percentage | 22.222% |
| overlapping transactions | 57.143% |
| waves | 4 |
| max wave width | 3 |
| critical path by tx | 4 |
| gas-weighted critical path | 405 gas covered |
| theoretical ceiling by tx | 1.750x |
| theoretical ceiling by gas | 1.593x |

Worker simulation reaches the gas critical-path bound at 2 workers in the demo. More workers do not help because observed dependencies are already binding.

## Real-Chain Collection

First check provider capability without tracing the full range:

```sh
cargo run -p parallel-revm-lab -- rpc-capability-check \
  --chain base \
  --block 38014901 \
  --rpc-url "$BASE_RPC_URL"
```

If the environment does not provide `BASE_RPC_URL`, the command fails before network access with a clear missing-URL message. If a provider lacks `debug_traceTransaction` or custom JavaScript tracer support, the command reports that capability gap without printing the RPC URL.

Collect the full block used by the committed case study:

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014901 \
  --rpc-url "$BASE_RPC_URL" \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-full \
  --resume
```

If that succeeds and the compact trace pack remains reviewable, optionally expand to the public range:

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

Use `--dry-run` first to check every requested block header and transaction count without tracing. RPC URLs are never printed, and live RPC collection is not required in CI.

Then analyze the collected trace pack:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-execution-dossier/dossier.json \
  --markdown case-studies/base-38014901-execution-dossier/executive-summary.md \
  --html case-studies/base-38014901-execution-dossier/dossier.html \
  --trace case-studies/base-38014901-execution-dossier/schedule.trace.json
```

Observed access hints can be generated with:

```sh
cargo run -p parallel-revm-lab -- recommend-access-lists \
  --trace-dir trace-packs/base-38014901-full \
  --out case-studies/base-38014901-execution-dossier/access-hints.json \
  --markdown case-studies/base-38014901-execution-dossier/access-hints.md
```

Run the trace-derived synthetic benchmark:

```sh
cargo run --release -p parallel-revm-lab -- bench-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --mode all \
  --threads 1,2,4,8,16 \
  --vm-steps-per-gas 1 \
  --out case-studies/base-38014901-execution-dossier/trace-derived-benchmark.json
```

## What Is Real Vs Synthetic

| Layer | Provenance | What it proves | What it does not prove |
| --- | --- | --- | --- |
| synthetic scheduler | generated workload | sequential equivalence of implemented schedulers | EVM semantics |
| demo trace pack | synthetic/demo fixture | dossier schema, gas-weighted scheduling, hot-state analysis | Base mainnet behavior |
| Geth tracer | optional RPC collection tool | compact storage observation path where providers support JS tracers | provider-wide trace compatibility |
| real Base block | user-collected RPC trace pack | real observed storage contention for Base block 38014901 | full-range representativeness or production throughput |
| revm smoke | real `revm` bytecode | inspector storage observations can feed trace packs and dossiers | full block replay |

## What It Demonstrates

- Deterministic state, access-key, delta, and stable hash model.
- Sequential baseline, access-list wave scheduler, and optimistic validation executor.
- Compact trace-pack schema with provenance, block files, gas, and validated EVM hex fields.
- Dossier analysis: conflict graph, overlapping-transaction percentage, waves, gas-weighted critical path, worker simulation, hot contracts, hot slots, concentration, and warnings.
- Optional Geth JS storage tracer, `rpc-capability-check`, and `collect-block-range` commands for user-collected RPC trace packs.
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
- `trace-packs/base-38014901-full`: committed compact real Base block trace pack.
- `case-studies/base-38014901-execution-dossier`: committed full-block real Base dossier.
- `case-studies/demo-trace-pack`: committed offline demo dossier.
- `case-studies/base-38014901-38014910`: full-range reproduction notes.

## What To Review In 90 Seconds

1. `case-studies/base-38014901-execution-dossier/executive-summary.md`
2. `case-studies/base-38014901-execution-dossier/optimization-memo.md`
3. `case-studies/base-38014901-execution-dossier/dossier.html`
4. `case-studies/demo-trace-pack/summary.md`
5. `crates/trace-model/src/trace_pack.rs`
6. `crates/analyzer/src/dossier.rs`
7. `crates/cli/src/main.rs`
8. `tracers/geth-storage-access-tracer.js`
