# Parallelism Report

Parallelism reports describe access contention, not production throughput.

## Core Fields

- `tx_count`: number of transactions in the normalized trace.
- `conflict_pair_count`: number of pairwise read/write or write/write conflicts.
- `conflict_percentage`: percentage of all possible transaction pairs that conflict.
- `overlapping_tx_percentage`: percentage of transactions that share at least one observed access key with another transaction.
- `wave_count`: number of deterministic dependency waves.
- `max_wave_width`: largest number of transactions in a single wave.
- `critical_path_length`: longest dependency chain.
- `theoretical_parallelism_ceiling`: `tx_count / critical_path_length`.
- `top_hot_contracts`: contracts with the most observed access events.
- `top_hot_storage_slots`: storage slots with the most observed access events.
- `deterministic_hash`: stable hash of the report content with the hash field cleared.

## Interpreting Waves

A wave is a set of transactions whose dependencies have already appeared in earlier waves. For complete traces, transactions in the same wave are conflict-free under this repo's access model. For incomplete-read traces, waves are lower-bound estimates and may be too wide.

High `max_wave_width` suggests available parallelism. High `wave_count` or critical path length suggests serial bottlenecks.

## Hot State

Hot contracts and slots are ranked by access count, with deterministic key tie-breaks. These tables answer where contention concentrates; they do not prove that a contract is slow, expensive, or badly designed.

## Report Artifacts

The fixture command writes:

- JSON: machine-readable report
- Markdown: reviewable text summary
- HTML: static visual summary with cards and tables
- Chrome trace JSON: deterministic schedule visualization

Open the Chrome trace JSON with `chrome://tracing` or compatible trace viewers.

## Trace-Pack Dossiers

Trace-pack dossiers extend the single-block report with range-level fields:

- `provenance`: whether data is a synthetic/demo fixture or user-collected RPC trace pack
- `source_tx_count` and `tx_coverage_percentage`: original transaction count and included coverage for partial samples
- `total_gas_used`: gas covered by included transactions, present only when gas is available for all included transactions and included gas reconciles with transaction gas
- `gas_weighted_conflict_percentage`: conflict percentage weighted by pair gas
- `gas_weighted_critical_path`: longest dependency path by gas duration
- `theoretical_parallelism_ceiling_by_gas`: `total_gas_covered / gas_weighted_critical_path`
- `worker_simulation`: deterministic list scheduling for requested worker counts
- `worst_serializing_txs`: highest conflict-degree transactions, tie-broken by duration and canonical position
- `top_hot_contracts` and `top_hot_storage_slots`: tx count, gas, unique slots, and conflict contribution
- `contention_concentration`: percent of conflict contributions caused by top 1/5/10 keys

Worker simulation reports a makespan in duration units, speedup versus one worker, idle percentage, and whether the schedule appears dependency-bound, worker-bound, or mixed. These are theoretical scheduling numbers, not measured execution throughput.

`overlapping_tx_percentage` is useful for public contention framing, but only as a same-family metric. Do not compare it as equal to another benchmark unless the trace scope, included transaction set, and access completeness model are the same.

## Current Real Sample

`case-studies/base-38014901-real-sample/` is a user-collected Base sample from block `38014901`.

- Coverage: 25 of 436 transactions, 5.734%.
- Conflict pairs: 1, 0.333%.
- Overlapping transactions: 22 of 25, 88.000%.
- Gas-weighted conflict percentage: 0.564%.
- Waves: 2, max width 24.
- Gas-weighted critical path: 563,160.
- Theoretical gas ceiling: 6.365x.

This is a real trace-backed storage-access sample, but it is partial and uses storage opcode observations only. It is the same metric family as public contention framing, not a full-block comparable measurement.

CSV sidecars are emitted for hot contracts, hot slots, and worker simulation.
