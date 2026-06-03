# Parallelism Report

Parallelism reports describe access contention, not production throughput.

## Core Fields

- `tx_count`: number of transactions in the normalized trace.
- `conflict_pair_count`: number of pairwise read/write or write/write conflicts.
- `conflict_percentage`: percentage of all possible transaction pairs that conflict.
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
