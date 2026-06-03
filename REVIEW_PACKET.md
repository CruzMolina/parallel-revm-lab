# parallel-revm-lab Review Packet

`parallel-revm-lab` is a Rust execution-contention lab that traces real Base EVM storage access, derives deterministic dependency graphs, simulates gas-weighted schedules, and compares scheduler policies without claiming production TPS.

## Why This Exists

Production Rust execution-client work is often private. This repository provides public evidence of Rust systems work, deterministic execution, concurrency-aware scheduling, benchmark discipline, EVM trace analysis, and technical writing around real and synthetic artifacts.

## Open First

- `case-studies/base-38014901-execution-dossier/executive-summary.md`
- `case-studies/base-38014901-execution-dossier/optimization-memo.md`
- `case-studies/base-38014901-execution-dossier/dossier.html`
- `crates/analyzer`
- `crates/trace-model`
- `crates/executor`
- `crates/revm-smoke`
- `tracers/geth-storage-access-tracer.js`

## Headline Real-Data Findings

| metric | value |
| --- | ---: |
| chain/block | Base `38014901` |
| txs analyzed | 436 of 436 |
| gas covered | 71,655,982 |
| observed storage/access events | 27,034 |
| unique contracts | 386 |
| unique storage slots | 4,321 |
| overlapping txs | 403 of 436, 92.431% |
| write-related conflict pairs | 5,052, 5.327% |
| waves | 60 |
| max wave width | 106 |
| gas-weighted critical path | 16,951,990 |
| theoretical gas ceiling | 4.227x |
| canonical 16-worker simulated speedup | 4.227x |
| best scheduler ablation | critical-path priority at 4 workers: 4.000x, 7.460% better than canonical |
| top hot contract | Base USDC, `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913` |
| top hot storage address | Base WETH, `0x4200000000000000000000000000000000000006` |

High observed overlap did not imply equivalent serialization; read-compatible overlap, write-related dependencies, and gas-weighted critical path are separated.

## Scheduler Ablation

The canonical ready-queue schedule reached 3.701x at 4 workers and 4.227x at 16 workers. Critical-path priority reached 4.000x at 4 workers, improving canonical by 7.460%, and reached the gas critical-path lower bound by 8 workers. Gas-LPT can hurt this graph because a long transaction is not always the most valuable next task; delaying a shorter transaction that unlocks a long dependency chain can increase the makespan.

These are theoretical gas-duration schedules over observed dependencies, not measured production throughput.

## Real Vs Synthetic

| Layer | Real? | What it proves | What it does not prove |
| --- | --- | --- | --- |
| full Base trace pack | real traced data | observed SLOAD/SSTORE topology | complete EVM semantics |
| scheduler analysis | deterministic model | dependency/wave/critical-path structure | production execution speed |
| synthetic executor | synthetic | sequential equivalence under parallel scheduling | full EVM state-transition replay |
| trace-derived benchmark | synthetic from real topology | timing behavior on the same access graph | TPS/Ggas/s |
| revm smoke | real revm bytecode | inspector bridge captures SLOAD/SSTORE | full EVM block/state replay |

## One-Command Local Review

```sh
just reviewer-demo
```

If `just` is unavailable:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --workers 1,2,4,8,16 \
  --out /tmp/parallel-revm-lab-review-dossier.json \
  --markdown /tmp/parallel-revm-lab-review-dossier.md \
  --html /tmp/parallel-revm-lab-review-dossier.html \
  --trace /tmp/parallel-revm-lab-review-schedule.trace.json

cargo run -p parallel-revm-lab -- verify \
  --workload mixed \
  --txs 25 \
  --conflicts 0.0,0.5 \
  --threads 1,2 \
  --seed-start 1 \
  --seed-end 2

cargo test -p parallel-revm-lab-revm-smoke --all-features
```

## Latest Validation Status

These commands passed in the latest packaging pass:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`: 73 tests plus doctests.
- `cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/base-38014901-full --workers 1,2,4,8,16 --out case-studies/base-38014901-execution-dossier/dossier.json --markdown case-studies/base-38014901-execution-dossier/executive-summary.md --html case-studies/base-38014901-execution-dossier/dossier.html --trace case-studies/base-38014901-execution-dossier/schedule.trace.json`
- `cargo run -p parallel-revm-lab -- recommend-access-lists --trace-dir trace-packs/base-38014901-full --out case-studies/base-38014901-execution-dossier/access-hints.json --markdown case-studies/base-38014901-execution-dossier/access-hints.md`
- `cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/demo-mini --workers 1,2,4,8 --out reports/demo-dossier.json --markdown reports/demo-dossier.md --html reports/demo-dossier.html --trace reports/demo-schedule.trace.json`
- `cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20`
- `cargo run -p parallel-revm-lab -- bench-trace-pack --trace-dir trace-packs/base-38014901-full --mode all --threads 1,2,4,8 --vm-steps-per-gas 1 --out case-studies/base-38014901-execution-dossier/trace-derived-bench.json`
- `cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json`: access-list 3.246x, optimistic 2.326x, state hash `1940c0cfba64e3cb`.
- `cargo test -p parallel-revm-lab-revm-smoke --all-features`: 4 tests.
- `just reviewer-demo`
- `just reviewer-validate`

## Caveats

- This is not a production execution client.
- This does not execute/replay full EVM state transitions.
- This does not claim production TPS or Ggas/s.
- Trace completeness depends on tracer and provider behavior.
- Scheduler results are theoretical.
- Contract labels are static convenience hints and are not part of analysis correctness.
