# Base 38014901 Real Sample

This case study is a real, user-collected Base trace-backed sample from block `38014901`.

It is not a full-block replay, not a production execution-client benchmark, and not a TPS or Ggas/s claim. It covers the first 25 transactions from a 436-transaction block using compact `SLOAD`/`SSTORE` observations from `debug_traceTransaction` with the repository's Geth JavaScript storage tracer.

## Headline Findings

| metric | value |
| --- | ---: |
| block | `38014901` |
| transactions covered | 25 of 436 (5.734%) |
| gas covered | 3,584,277 |
| observed storage accesses | 680 |
| unique contracts | 62 |
| unique storage slots | 226 |
| conflict pairs | 1 (0.333%) |
| gas-weighted conflict percentage | 0.564% |
| overlapping transactions | 22 (88.000%) |
| waves | 2 |
| max wave width | 24 |
| tx-count critical path | 2 |
| gas-weighted critical path | 563,160 |
| theoretical ceiling by tx | 12.500x |
| theoretical ceiling by gas | 6.365x |
| 16-worker simulated speedup | 6.168x |
| top hot contract | `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789` |
| top conflict slot | `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789:0x0000000000000000000000000000000000000000000000000000000000000002` |

The sample has high observed key overlap but only one write-related conflict pair under this repository's access model. That means many transactions touch the same observed storage keys in read-compatible ways, while one pair creates a real dependency edge.

## Files

- `executive-summary.md`: generated dossier summary.
- `dossier.json`: full machine-readable report.
- `dossier.html`: static report.
- `schedule.trace.json`: Chrome trace schedule view.
- `hot-contracts.csv`, `hot-slots.csv`, `worker-simulation.csv`: compact tables.
- `access-hints.json`, `access-hints.md`: observed access hints, not complete production access lists.
- `optimization-memo.md`: human-written engineering memo.
- `provenance.md`: collection details and limitations.

## Reproduce

```sh
cargo run -p parallel-revm-lab -- rpc-capability-check \
  --chain base \
  --block 38014901

cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014901 \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-real-sample \
  --max-transactions 25 \
  --resume

cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-real-sample \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-real-sample/dossier.json \
  --markdown case-studies/base-38014901-real-sample/executive-summary.md \
  --html case-studies/base-38014901-real-sample/dossier.html \
  --trace case-studies/base-38014901-real-sample/schedule.trace.json

cargo run -p parallel-revm-lab -- recommend-access-lists \
  --trace-dir trace-packs/base-38014901-real-sample \
  --out case-studies/base-38014901-real-sample/access-hints.json \
  --markdown case-studies/base-38014901-real-sample/access-hints.md
```

Set `BASE_RPC_URL` or `ETH_RPC_URL` in the shell. Do not paste RPC URLs into committed files.
