# Contention Dossier: synthetic-base-shaped blocks 900000001-900000002

**Provenance:** `synthetic-base-shaped demo fixture (not real Base data)`

**Tracer:** `sanitized-geth-js-storage`

## Summary

- Blocks: 2
- Transactions: 7
- Accesses: 7
- Conflict pairs: 2 (22.222%)
- Overlapping transactions: 4 (57.143%)
- Overlap is broader than serialization: read-compatible overlap can be high even when write-dependent conflict pairs and waves stay low.
- Waves: 4
- Max wave width: 3
- Critical path by tx count: 4
- Theoretical ceiling by tx: 1.750x
- Total gas covered: 645
- Gas-weighted critical path: 405
- Theoretical ceiling by gas: 1.593x

## Worker Simulation

| workers | makespan | speedup | idle | interpretation |
| ---: | ---: | ---: | ---: | --- |
| 1 | 645 | 1.000x | 0.00% | worker-bound: workers stay mostly occupied under observed dependencies |
| 2 | 405 | 1.593x | 20.37% | dependency-bound: makespan is at the critical-path lower bound |
| 4 | 405 | 1.593x | 60.19% | dependency-bound: makespan is at the critical-path lower bound |
| 8 | 405 | 1.593x | 80.09% | dependency-bound: makespan is at the critical-path lower bound |

## Scheduler Ablation

All strategies preserve observed dependencies; they only change deterministic ready-queue priority.

| strategy | workers | makespan | speedup vs canonical 1-worker | idle | ready wait | critical-path bound | improvement vs canonical | deps preserved |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `canonical` | 1 | 645 | 1.000x | 0.00% | 620 | 405 | 0.00% | true |
| `canonical` | 2 | 405 | 1.593x | 20.37% | 50 | 405 | 0.00% | true |
| `canonical` | 4 | 405 | 1.593x | 60.19% | 0 | 405 | 0.00% | true |
| `canonical` | 8 | 405 | 1.593x | 80.09% | 0 | 405 | 0.00% | true |
| `gas_lpt` | 1 | 645 | 1.000x | 0.00% | 630 | 405 | 0.00% | true |
| `gas_lpt` | 2 | 405 | 1.593x | 20.37% | 80 | 405 | 0.00% | true |
| `gas_lpt` | 4 | 405 | 1.593x | 60.19% | 0 | 405 | 0.00% | true |
| `gas_lpt` | 8 | 405 | 1.593x | 80.09% | 0 | 405 | 0.00% | true |
| `critical_path` | 1 | 645 | 1.000x | 0.00% | 720 | 405 | 0.00% | true |
| `critical_path` | 2 | 405 | 1.593x | 20.37% | 80 | 405 | 0.00% | true |
| `critical_path` | 4 | 405 | 1.593x | 60.19% | 0 | 405 | 0.00% | true |
| `critical_path` | 8 | 405 | 1.593x | 80.09% | 0 | 405 | 0.00% | true |

## Worst Serializing Transactions

| block | tx | wave | conflicts | duration | tx hash |
| ---: | ---: | ---: | ---: | ---: | --- |
| 900000001 | 2 | 1 | 1 | 120 | `0x0000000000000000000000000000000000000000000000000000000000000102` |
| 900000001 | 0 | 0 | 1 | 100 | `0x0000000000000000000000000000000000000000000000000000000000000100` |
| 900000002 | 2 | 1 | 1 | 95 | `0x0000000000000000000000000000000000000000000000000000000000000202` |
| 900000002 | 0 | 0 | 1 | 90 | `0x0000000000000000000000000000000000000000000000000000000000000200` |

## Hot Contracts

Labels are convenience metadata for readability; they are not part of analysis correctness.

| contract | label | txs | unique slots | gas covered | conflict contribution |
| --- | --- | ---: | ---: | ---: | ---: |
| `0x1111111111111111111111111111111111111111` | unknown | 4 | 2 | 410 | 1 |
| `0x3333333333333333333333333333333333333333` | unknown | 2 | 1 | 185 | 1 |
| `0x2222222222222222222222222222222222222222` | unknown | 1 | 1 | 50 | 0 |

## Hot Storage Slots

| slot | address label | txs | gas covered | conflict contribution |
| --- | --- | ---: | ---: | ---: |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000000` | unknown | 3 | 330 | 1 |
| `0x3333333333333333333333333333333333333333:0x0000000000000000000000000000000000000000000000000000000000000007` | unknown | 2 | 185 | 1 |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000002` | unknown | 1 | 80 | 0 |
| `0x2222222222222222222222222222222222222222:0x0000000000000000000000000000000000000000000000000000000000000001` | unknown | 1 | 50 | 0 |

## Warning Summary

- 7 of 7 analyzed txs: read coverage is incomplete
- Synthetic demo fixture; do not cite as real-chain evidence
- block 900000001: Block uses a synthetic number and is not a Base RPC capture
- block 900000002: Block uses a synthetic number and is not a Base RPC capture
- gas-weighted scheduling is theoretical, not measured throughput

Full per-transaction warnings are preserved in `dossier.json`.

## What This Proves

This dossier shows deterministic access-contention structure, hot-state concentration, gas-weighted theoretical scheduling bounds where gas is available, and worker-count sensitivity for the supplied trace pack.

## What This Does Not Prove

It is not production TPS, not Ggas/s, does not execute/replay full EVM state transitions, and is not proof that observed access hints are complete Ethereum access lists.
