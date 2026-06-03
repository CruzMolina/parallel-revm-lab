# Base Block 38014901 Execution Contention Dossier

This case study analyzes one real Base block with compact normalized traces collected through `debug_traceTransaction` and the repository's Geth JavaScript storage tracer.

## Headline Findings

- Dataset: Base block `38014901`, 436 of 436 transactions covered.
- Gas covered: 71,655,982.
- Observed accesses: 27,034 across 386 contracts and 4,321 storage slots.
- Conflict pairs: 5,052 of 94,830 possible pairs, 5.327%.
- Overlapping transactions: 403 of 436, 92.431%.
- Gas-weighted conflict percentage: 7.172%.
- Waves: 60, max wave width 106.
- Gas-weighted critical path: 16,951,990 gas units.
- Theoretical gas ceiling: 4.227x.
- Canonical 16-worker simulated speedup: 4.227x, essentially at the gas critical-path bound.
- Top hot contract by conflict contribution: `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913`.
- Top hot storage slot by conflict contribution: `0x4200000000000000000000000000000000000006:0x0d52ad225b9f8da090dc37c741705dabc30f648dce00d7b0cab66994a1261ea6`.

High overlap does not mean the block is mostly serialized. In this block, many transactions share at least one observed access key, but the write-dependent conflict graph leaves substantial wave-level parallelism.

## Artifacts

- `executive-summary.md`: generated Markdown dossier.
- `dossier.json`: full machine-readable metrics, warning detail, worker simulation, and scheduler ablation.
- `dossier.html`: standalone HTML report.
- `schedule.trace.json`: Chrome trace schedule visualization.
- `hot-contracts.csv`: top contracts by observed contention.
- `hot-slots.csv`: top storage slots by observed contention.
- `worker-simulation.csv`: canonical worker simulation for 1/2/4/8/16 workers.
- `scheduler-ablation.csv`: canonical, gas-LPT, and critical-path scheduler comparison.
- `access-hints.json` and `access-hints.md`: observed access hints, not production access lists.
- `trace-derived-benchmark.json`: synthetic execution benchmark that reuses observed access topology.
- `optimization-memo.md`: short engineering interpretation and next steps.
- `provenance.md`: collection command shape, limitations, and data handling notes.

## Caveats

This is a Rust protocol-engineering case study over observed SLOAD/SSTORE-style storage traces. It is not production client throughput, not Ggas/s, not full EVM replay, and not a complete Ethereum access-list generator.
