# Benchmarks

Benchmarks in this repository measure synthetic scheduler/workbench throughput. They are not full-node TPS, gas throughput, or production blockchain-client benchmarks.

## Smoke Command

```sh
cargo run --release -p parallel-revm-lab -- bench \
  --workload mixed \
  --txs 1000 \
  --conflict 0.5 \
  --mode all \
  --threads 4 \
  --seed 42 \
  --out reports/mixed-c50.json \
  --trace reports/mixed-c50.trace.json
```

## Scenario Matrix

The `just bench-all` recipe runs:

- `erc20`: `c0`, `c20`, `c50`
- `hot-pool`: `c70`, `c95`
- `mixed`: `c20`, `c50`

Each scenario writes a JSON report under `reports/`.

## Interpreting Reports

Report fields include:

- requested and observed conflict pressure
- mode elapsed time
- synthetic tx/s
- speedup relative to sequential where a sequential baseline is present
- final state hash
- detected conflicts, wave counts, and optimistic re-execution counts
- deterministic pass/fail status

Speedup is meaningful only within the same report and machine. High-contention workloads are expected to degrade because fewer transactions can safely execute or commit without re-execution.

## Latest Local Snapshot

Recorded from `reports/mixed-c50.json` after running the smoke command in release mode.

Environment:

- `rustc 1.92.0 (ded5c06cf 2025-12-08)`
- macOS Darwin 25.5.0 arm64
- Apple M2 Pro, 12 logical CPUs

Workload:

- `mixed`
- `tx_count`: 1000
- requested conflict: `0.5`
- observed conflict: `0.12498098098098098`
- seed: `42`
- threads: `4`

| mode | elapsed ns | synthetic tx/s | speedup vs sequential | state hash | key scheduler metric |
| --- | ---: | ---: | ---: | --- | --- |
| sequential | 649,000 | 1,540,832.05 | baseline | `ac90d19c91175700` | input-order baseline |
| access-list | 59,640,584 | 16,767.11 | 0.011x | `ac90d19c91175700` | 309 waves, max width 245 |
| optimistic | 793,042 | 1,260,967.26 | 0.818x | `ac90d19c91175700` | 731 re-executions |

All modes reported `deterministic_passed: true`. The access-list result is intentionally not dressed up: for this small synthetic batch, scheduling overhead dominates. The report is still useful because it shows honest degradation while preserving the sequential state hash.
