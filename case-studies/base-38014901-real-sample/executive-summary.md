# Contention Dossier: base blocks 38014901-38014901

**Provenance:** `user-collected RPC trace pack`

**Tracer:** `geth-js-storage`

## Summary

- Blocks: 1
- Transactions: 25
- Source transactions covered: 25 of 436 (5.734%)
- Accesses: 680
- Conflict pairs: 1 (0.333%)
- Overlapping transactions: 22 (88.000%)
- Waves: 2
- Max wave width: 24
- Critical path by tx count: 2
- Theoretical ceiling by tx: 12.500x
- Total gas covered: 3584277
- Gas-weighted critical path: 563160
- Theoretical ceiling by gas: 6.365x

## Worker Simulation

| workers | makespan | speedup | idle | interpretation |
| ---: | ---: | ---: | ---: | --- |
| 1 | 3584277 | 1.000x | 0.00% | worker-bound: workers stay mostly occupied under observed dependencies |
| 2 | 2002461 | 1.790x | 10.50% | mixed dependency/worker-bound: dependencies and idle capacity both matter |
| 4 | 1181384 | 3.034x | 24.15% | mixed dependency/worker-bound: dependencies and idle capacity both matter |
| 8 | 760948 | 4.710x | 41.12% | mixed dependency/worker-bound: dependencies and idle capacity both matter |
| 16 | 581067 | 6.168x | 61.45% | mixed dependency/worker-bound: dependencies and idle capacity both matter |

## Worst Serializing Transactions

| block | tx | wave | conflicts | duration | tx hash |
| ---: | ---: | ---: | ---: | ---: | --- |
| 38014901 | 24 | 1 | 1 | 309838 | `0x0836dca8f01c33b320e570ea8a47687ca0a5f88a57af44d9cef3d7c9f30c9515` |
| 38014901 | 23 | 0 | 1 | 175552 | `0x009cd9a5c35c851ce4e38e94b7b1ed70aa2643ff040a7e72ddd4701ccf29da8d` |

## Hot Contracts

| contract | txs | unique slots | gas covered | conflict contribution |
| --- | ---: | ---: | ---: | ---: |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789` | 2 | 4 | 485390 | 2 |
| `0x976a4d13a41581c5eda72c493038ac10cadf5e46` | 10 | 1 | 1189217 | 0 |
| `0x20cb8f872ae894f7c9e32e621c186e5afce82fd0` | 9 | 2 | 880556 | 0 |
| `0x22aee3699b6a0fed71490c103bd4e5f3309891d5` | 9 | 1 | 979811 | 0 |
| `0x57135dd4b832645955422b28291302d697ea0900` | 8 | 1 | 671150 | 0 |
| `0x498581ff718922c3f8e6a244956af099b2652b2b` | 7 | 6 | 847377 | 0 |
| `0x0f12cb1f3e37375af09a248ca97c4a3eedf2f494` | 6 | 1 | 637971 | 0 |
| `0x4c9f68e780523feb4c9bb1aad2e5cc3b6476892b` | 6 | 1 | 637971 | 0 |
| `0x5b8bf0cd0fa5bf970ebe558d7551a668dadf3570` | 6 | 1 | 787046 | 0 |
| `0x7631e2b08317f227bb2916ed0e9d69c64d73bdcd` | 6 | 1 | 687791 | 0 |

## Hot Storage Slots

| slot | txs | gas covered | conflict contribution |
| --- | ---: | ---: | ---: |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789:0x0000000000000000000000000000000000000000000000000000000000000002` | 2 | 485390 | 1 |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789:0x33a26eb216320a51847a061c8f256bd98ae96e1a1d67d2bbc4cd8c61adad2df0` | 2 | 485390 | 1 |
| `0x976a4d13a41581c5eda72c493038ac10cadf5e46:0x0000000000000000000000000000000000000000000000000000000000000006` | 10 | 1189217 | 0 |
| `0x20cb8f872ae894f7c9e32e621c186e5afce82fd0:0x0000000000000000000000000000000000000000000000000000000000000000` | 9 | 880556 | 0 |
| `0x20cb8f872ae894f7c9e32e621c186e5afce82fd0:0x0000000000000000000000000000000000000000000000000000000000000001` | 9 | 880556 | 0 |
| `0x22aee3699b6a0fed71490c103bd4e5f3309891d5:0x0000000000000000000000000000000000000000000000000000000000000006` | 9 | 979811 | 0 |
| `0x57135dd4b832645955422b28291302d697ea0900:0x0000000000000000000000000000000000000000000000000000000000000006` | 8 | 671150 | 0 |
| `0x0f12cb1f3e37375af09a248ca97c4a3eedf2f494:0x0000000000000000000000000000000000000000000000000000000000000000` | 6 | 637971 | 0 |
| `0x498581ff718922c3f8e6a244956af099b2652b2b:0x934ff9aed835597de93bbfd6474d60d33c1e72493b62f8e1615cfda0908eeaef` | 6 | 637971 | 0 |
| `0x4c9f68e780523feb4c9bb1aad2e5cc3b6476892b:0x0000000000000000000000000000000000000000000000000000000000000000` | 6 | 637971 | 0 |

## Warnings

- Provider trace completeness varies; verify tracer support before making claims
- block 38014901: collection truncated to 25 of 436 transactions by --max-transactions
- block 38014901: trace pack access observations may be incomplete; gas-weighted scheduling is theoretical
- block 38014901: tx_index 0: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 0: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 0: trace marks read information as incomplete
- block 38014901: tx_index 10: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 10: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 10: trace marks read information as incomplete
- block 38014901: tx_index 11: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 11: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 11: trace marks read information as incomplete
- block 38014901: tx_index 12: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 12: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 12: trace marks read information as incomplete
- block 38014901: tx_index 13: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 13: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 13: trace marks read information as incomplete
- block 38014901: tx_index 14: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 14: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 14: trace marks read information as incomplete
- block 38014901: tx_index 15: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 15: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 15: trace marks read information as incomplete
- block 38014901: tx_index 16: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 16: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 16: trace marks read information as incomplete
- block 38014901: tx_index 17: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 17: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 17: trace marks read information as incomplete
- block 38014901: tx_index 18: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 18: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 18: trace marks read information as incomplete
- block 38014901: tx_index 19: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 19: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 19: trace marks read information as incomplete
- block 38014901: tx_index 1: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 1: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 1: trace marks read information as incomplete
- block 38014901: tx_index 20: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 20: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 20: trace marks read information as incomplete
- block 38014901: tx_index 21: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 21: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 21: trace marks read information as incomplete
- block 38014901: tx_index 22: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 22: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 22: trace marks read information as incomplete
- block 38014901: tx_index 23: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 23: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 23: debug tracer reported 5 fault(s)
- block 38014901: tx_index 23: trace marks read information as incomplete
- block 38014901: tx_index 24: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 24: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 24: trace marks read information as incomplete
- block 38014901: tx_index 2: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 2: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 2: trace marks read information as incomplete
- block 38014901: tx_index 3: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 3: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 3: trace marks read information as incomplete
- block 38014901: tx_index 4: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 4: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 4: trace marks read information as incomplete
- block 38014901: tx_index 5: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 5: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 5: trace marks read information as incomplete
- block 38014901: tx_index 6: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 6: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 6: debug tracer reported 8 fault(s)
- block 38014901: tx_index 6: trace marks read information as incomplete
- block 38014901: tx_index 7: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 7: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 7: trace marks read information as incomplete
- block 38014901: tx_index 8: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 8: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 8: trace marks read information as incomplete
- block 38014901: tx_index 9: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- block 38014901: tx_index 9: Provider support for debug_traceTransaction and JavaScript tracers varies.
- block 38014901: tx_index 9: trace marks read information as incomplete

## What This Proves

This dossier shows deterministic access-contention structure, hot-state concentration, gas-weighted theoretical scheduling bounds where gas is available, and worker-count sensitivity for the supplied trace pack.

## What This Does Not Prove

It is not production TPS, not Ggas/s, not full block replay, and not proof that observed access hints are complete Ethereum access lists.
