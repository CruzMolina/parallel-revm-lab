# Contention Dossier: revm-smoke blocks 0-0

**Provenance:** `real revm bytecode execution over tiny in-memory fixtures`

**Tracer:** `revm-inspector-sload-sstore`

## Summary

- Blocks: 1
- Transactions: 3
- Source transactions covered: 3 of 3 (100.000%)
- Accesses: 5
- Conflict pairs: 1 (33.333%)
- Overlapping transactions: 2 (66.667%)
- Waves: 2
- Max wave width: 2
- Critical path by tx count: 2
- Theoretical ceiling by tx: 1.500x
- Gas-weighted metrics: unavailable because gas is missing

## Worker Simulation

| workers | makespan | speedup | idle | interpretation |
| ---: | ---: | ---: | ---: | --- |
| 1 | 3 | 1.000x | 0.00% | worker-bound: workers stay mostly occupied under observed dependencies |
| 2 | 2 | 1.500x | 25.00% | dependency-bound: makespan is at the critical-path lower bound |
| 4 | 2 | 1.500x | 62.50% | dependency-bound: makespan is at the critical-path lower bound |

## Worst Serializing Transactions

| block | tx | wave | conflicts | duration | tx hash |
| ---: | ---: | ---: | ---: | ---: | --- |
| 0 | 0 | 0 | 1 | 1 | `0x0000000000000000000000000000000000000000000000000000000000000000` |
| 0 | 2 | 1 | 1 | 1 | `0x0000000000000000000000000000000000000000000000000000000000000002` |

## Hot Contracts

| contract | txs | unique slots | gas covered | conflict contribution |
| --- | ---: | ---: | ---: | ---: |
| `0x2222222222222222222222222222222222222222` | 3 | 2 | unavailable | 1 |

## Hot Storage Slots

| slot | txs | gas covered | conflict contribution |
| --- | ---: | ---: | ---: |
| `0x2222222222222222222222222222222222222222:0x0000000000000000000000000000000000000000000000000000000000000007` | 2 | unavailable | 1 |
| `0x2222222222222222222222222222222222222222:0x0000000000000000000000000000000000000000000000000000000000000008` | 1 | unavailable | 0 |

## Warnings

- Storage opcode observations only; account/code/balance reads are not represented
- block 0: revm smoke inspector records SLOAD/SSTORE storage access only; account, balance, nonce, and code reads are not represented
- block 0: trace pack access observations may be incomplete; gas-weighted scheduling is theoretical
- block 0: tx_index 0: revm smoke inspector marks reads incomplete outside SLOAD
- block 0: tx_index 0: trace marks read information as incomplete
- block 0: tx_index 1: revm smoke inspector marks reads incomplete outside SLOAD
- block 0: tx_index 1: trace marks read information as incomplete
- block 0: tx_index 2: revm smoke inspector marks reads incomplete outside SLOAD
- block 0: tx_index 2: trace marks read information as incomplete
- one or more blocks/transactions are missing gas; gas-weighted range metrics are unavailable

## What This Proves

This dossier shows deterministic access-contention structure, hot-state concentration, gas-weighted theoretical scheduling bounds where gas is available, and worker-count sensitivity for the supplied trace pack.

## What This Does Not Prove

It is not production TPS, not Ggas/s, not full block replay, and not proof that observed access hints are complete Ethereum access lists.
