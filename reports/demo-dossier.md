# Contention Dossier: synthetic-base-shaped blocks 900000001-900000002

**Provenance:** `synthetic-base-shaped demo fixture (not real Base data)`

**Tracer:** `sanitized-geth-js-storage`

## Summary

- Blocks: 2
- Transactions: 7
- Accesses: 7
- Conflict pairs: 2 (22.222%)
- Overlapping transactions: 4 (57.143%)
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

## Worst Serializing Transactions

| block | tx | wave | conflicts | duration | tx hash |
| ---: | ---: | ---: | ---: | ---: | --- |
| 900000001 | 2 | 1 | 1 | 120 | `0x0000000000000000000000000000000000000000000000000000000000000102` |
| 900000001 | 0 | 0 | 1 | 100 | `0x0000000000000000000000000000000000000000000000000000000000000100` |
| 900000002 | 2 | 1 | 1 | 95 | `0x0000000000000000000000000000000000000000000000000000000000000202` |
| 900000002 | 0 | 0 | 1 | 90 | `0x0000000000000000000000000000000000000000000000000000000000000200` |

## Hot Contracts

| contract | txs | unique slots | gas covered | conflict contribution |
| --- | ---: | ---: | ---: | ---: |
| `0x1111111111111111111111111111111111111111` | 4 | 2 | 410 | 1 |
| `0x3333333333333333333333333333333333333333` | 2 | 1 | 185 | 1 |
| `0x2222222222222222222222222222222222222222` | 1 | 1 | 50 | 0 |

## Hot Storage Slots

| slot | txs | gas covered | conflict contribution |
| --- | ---: | ---: | ---: |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000000` | 3 | 330 | 1 |
| `0x3333333333333333333333333333333333333333:0x0000000000000000000000000000000000000000000000000000000000000007` | 2 | 185 | 1 |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000002` | 1 | 80 | 0 |
| `0x2222222222222222222222222222222222222222:0x0000000000000000000000000000000000000000000000000000000000000001` | 1 | 50 | 0 |

## Warnings

- Synthetic demo fixture; do not cite as real-chain evidence.
- block 900000001: Block uses a synthetic number and is not a Base RPC capture.
- block 900000001: trace pack access observations may be incomplete; gas-weighted scheduling is theoretical
- block 900000001: tx_index 0: trace marks read information as incomplete
- block 900000001: tx_index 1: trace marks read information as incomplete
- block 900000001: tx_index 2: trace marks read information as incomplete
- block 900000001: tx_index 3: trace marks read information as incomplete
- block 900000002: Block uses a synthetic number and is not a Base RPC capture.
- block 900000002: trace pack access observations may be incomplete; gas-weighted scheduling is theoretical
- block 900000002: tx_index 0: trace marks read information as incomplete
- block 900000002: tx_index 1: trace marks read information as incomplete
- block 900000002: tx_index 2: trace marks read information as incomplete

## What This Proves

This dossier shows deterministic access-contention structure, hot-state concentration, gas-weighted theoretical scheduling bounds where gas is available, and worker-count sensitivity for the supplied trace pack.

## What This Does Not Prove

It is not production TPS, not Ggas/s, not full block replay, and not proof that observed access hints are complete Ethereum access lists.
