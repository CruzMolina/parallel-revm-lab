# Demo Trace-Pack Dossier

This case study is generated from `trace-packs/demo-mini`, a tiny synthetic/demo trace pack. It is not real Base data.

## Reproduce

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/demo-mini \
  --workers 1,2,4,8 \
  --out case-studies/demo-trace-pack/dossier.json \
  --markdown case-studies/demo-trace-pack/summary.md \
  --html case-studies/demo-trace-pack/dossier.html \
  --trace case-studies/demo-trace-pack/schedule.trace.json
```

## Current Summary

- Blocks: 2
- Transactions: 7
- Conflict pairs: 2 (22.222%)
- Critical path by tx count: 4
- Gas-weighted critical path: 405
- Theoretical ceiling by tx: 1.750x
- Theoretical ceiling by gas: 1.593x

Worker simulation reaches the gas critical-path bound at 2 workers. Additional workers do not improve this tiny demo because the observed dependency chain is already binding.

## Files

- `summary.md`: concise Markdown dossier.
- `dossier.json`: full machine-readable report.
- `dossier.html`: static human-readable report.
- `schedule.trace.json`: Chrome trace schedule view.
- `hot-contracts.csv`, `hot-slots.csv`, `worker-simulation.csv`: tables for quick inspection.

## Caveat

This proves the toolchain and report semantics. It does not prove anything about Base mainnet contention, throughput, Ggas/s, or production access-list completeness.
