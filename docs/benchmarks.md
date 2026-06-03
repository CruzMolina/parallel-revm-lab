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

## Heavier Work Command

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

`--vm-steps` adds deterministic CPU work to every synthetic transaction. It is useful for showing when scheduler overhead is amortized by heavier execution. It does not model EVM gas, opcode mix, storage latency, or production client throughput.

## Scenario Matrix

The `just bench-all` recipe runs:

- `erc20`: `c0`, `c20`, `c50`
- `hot-pool`: `c70`, `c95`
- `mixed`: `c20`, `c50`
- `storage c0` with `--vm-steps 50000`

Each scenario writes a JSON report under `reports/`.

## Interpreting Reports

Report fields include:

- requested and observed conflict pressure
- mode elapsed time
- synthetic tx/s
- speedup relative to sequential where a sequential baseline is present
- final state hash
- `declared_conflict_pairs`: pairwise conflicts implied by declared access sets
- `scheduler_deferrals`: access-list deferral decisions while building waves; this can exceed declared conflict pairs when a transaction is deferred across multiple waves
- `validation_failures`: optimistic speculative reads invalidated during canonical validation
- `reexecuted_txs`: optimistic transactions re-executed after validation failure
- `wave_count` and `max_wave_width`
- deterministic pass/fail status

Speedup is meaningful only within the same report and machine. High-contention workloads are expected to degrade because fewer transactions can safely execute or commit without re-execution. Cheap transactions can also degrade because scheduler overhead dominates.

## Latest Local Snapshots

Recorded after running the commands above in release mode.

Environment:

- `rustc 1.92.0 (ded5c06cf 2025-12-08)`
- macOS Darwin 25.5.0 arm64
- Apple M2 Pro, 12 logical CPUs

Workload:

- `mixed`
- `tx_count`: 1000
- requested conflict: `0.5`
- observed conflict: `0.12498098098098098`
- `vm_steps`: 0
- seed: `42`
- threads: `4`

Cheap mixed transactions (`reports/mixed-c50.json`):

| mode | elapsed ns | synthetic tx/s | speedup vs sequential | declared conflicts | scheduler/validation metric |
| --- | ---: | ---: | ---: | --- | --- |
| sequential | 739,166 | 1,352,876.08 | baseline | 62,428 | input-order baseline |
| access-list | 55,963,583 | 17,868.76 | 0.013x | 62,428 | 83,755 deferrals; 309 waves, max width 245 |
| optimistic | 879,500 | 1,137,009.66 | 0.840x | 62,428 | 731 validation failures and re-executions |

All modes reported `deterministic_passed: true` and state hash `ac90d19c91175700`. The access-list result is intentionally not dressed up: for this cheap synthetic batch, scheduling overhead dominates.

Heavier low-contention storage transactions (`reports/storage-c0-vmsteps.json`):

- `workload`: `storage`
- `tx_count`: 1000
- requested conflict: `0.0`
- observed conflict: `0.0004904904904904905`
- `vm_steps`: 50000
- seed: `42`
- threads: `4`

| mode | elapsed ns | synthetic tx/s | speedup vs sequential | declared conflicts | scheduler/validation metric |
| --- | ---: | ---: | ---: | --- | --- |
| sequential | 92,910,708 | 10,763.02 | baseline | 245 | input-order baseline |
| access-list | 28,748,042 | 34,784.98 | 3.232x | 245 | 245 deferrals; 4 waves, max width 793 |
| optimistic | 43,135,833 | 23,182.58 | 2.154x | 245 | 173 validation failures and re-executions |

All modes reported `deterministic_passed: true` and state hash `1940c0cfba64e3cb`. This scenario shows that parallel execution can win when deterministic synthetic execution work is heavy enough and declared contention is low.
