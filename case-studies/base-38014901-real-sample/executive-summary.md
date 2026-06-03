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
- Overlap is broader than serialization: read-compatible overlap can be high even when write-dependent conflict pairs and waves stay low.
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

## Scheduler Ablation

All strategies preserve observed dependencies; they only change deterministic ready-queue priority.

| strategy | workers | makespan | speedup vs canonical 1-worker | idle | ready wait | critical-path bound | improvement vs canonical | deps preserved |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `canonical` | 1 | 3584277 | 1.000x | 0.00% | 41012734 | 563160 | 0.00% | true |
| `canonical` | 2 | 2002461 | 1.790x | 10.50% | 19321248 | 563160 | 0.00% | true |
| `canonical` | 4 | 1181384 | 3.034x | 24.15% | 8547257 | 563160 | 0.00% | true |
| `canonical` | 8 | 760948 | 4.710x | 41.12% | 2668930 | 563160 | 0.00% | true |
| `canonical` | 16 | 581067 | 6.168x | 61.45% | 566808 | 563160 | 0.00% | true |
| `gas_lpt` | 1 | 3584277 | 1.000x | 0.00% | 59876415 | 563160 | 0.00% | true |
| `gas_lpt` | 2 | 1793885 | 1.998x | 0.10% | 28671856 | 563160 | 10.42% | true |
| `gas_lpt` | 4 | 905480 | 3.958x | 1.04% | 13118331 | 563160 | 23.35% | true |
| `gas_lpt` | 8 | 563160 | 6.365x | 20.44% | 4539256 | 563160 | 25.99% | true |
| `gas_lpt` | 16 | 563160 | 6.365x | 60.22% | 646156 | 563160 | 3.08% | true |
| `critical_path` | 1 | 3584277 | 1.000x | 0.00% | 60701974 | 563160 | 0.00% | true |
| `critical_path` | 2 | 1805167 | 1.986x | 0.72% | 29428570 | 563160 | 9.85% | true |
| `critical_path` | 4 | 920966 | 3.892x | 2.70% | 13608232 | 563160 | 22.04% | true |
| `critical_path` | 8 | 563160 | 6.365x | 20.44% | 4539256 | 563160 | 25.99% | true |
| `critical_path` | 16 | 563160 | 6.365x | 60.22% | 646156 | 563160 | 3.08% | true |

## Worst Serializing Transactions

| block | tx | wave | conflicts | duration | tx hash |
| ---: | ---: | ---: | ---: | ---: | --- |
| 38014901 | 24 | 1 | 1 | 309838 | `0x0836dca8f01c33b320e570ea8a47687ca0a5f88a57af44d9cef3d7c9f30c9515` |
| 38014901 | 23 | 0 | 1 | 175552 | `0x009cd9a5c35c851ce4e38e94b7b1ed70aa2643ff040a7e72ddd4701ccf29da8d` |

## Hot Contracts

Labels are convenience metadata for readability; they are not part of analysis correctness.

| contract | label | txs | unique slots | gas covered | conflict contribution |
| --- | --- | ---: | ---: | ---: | ---: |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789` | unknown | 2 | 4 | 485390 | 2 |
| `0x976a4d13a41581c5eda72c493038ac10cadf5e46` | unknown | 10 | 1 | 1189217 | 0 |
| `0x20cb8f872ae894f7c9e32e621c186e5afce82fd0` | unknown | 9 | 2 | 880556 | 0 |
| `0x22aee3699b6a0fed71490c103bd4e5f3309891d5` | unknown | 9 | 1 | 979811 | 0 |
| `0x57135dd4b832645955422b28291302d697ea0900` | unknown | 8 | 1 | 671150 | 0 |
| `0x498581ff718922c3f8e6a244956af099b2652b2b` | unknown | 7 | 6 | 847377 | 0 |
| `0x0f12cb1f3e37375af09a248ca97c4a3eedf2f494` | unknown | 6 | 1 | 637971 | 0 |
| `0x4c9f68e780523feb4c9bb1aad2e5cc3b6476892b` | unknown | 6 | 1 | 637971 | 0 |
| `0x5b8bf0cd0fa5bf970ebe558d7551a668dadf3570` | unknown | 6 | 1 | 787046 | 0 |
| `0x7631e2b08317f227bb2916ed0e9d69c64d73bdcd` | unknown | 6 | 1 | 687791 | 0 |

## Hot Storage Slots

| slot | address label | txs | gas covered | conflict contribution |
| --- | --- | ---: | ---: | ---: |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789:0x0000000000000000000000000000000000000000000000000000000000000002` | unknown | 2 | 485390 | 1 |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789:0x33a26eb216320a51847a061c8f256bd98ae96e1a1d67d2bbc4cd8c61adad2df0` | unknown | 2 | 485390 | 1 |
| `0x976a4d13a41581c5eda72c493038ac10cadf5e46:0x0000000000000000000000000000000000000000000000000000000000000006` | unknown | 10 | 1189217 | 0 |
| `0x20cb8f872ae894f7c9e32e621c186e5afce82fd0:0x0000000000000000000000000000000000000000000000000000000000000000` | unknown | 9 | 880556 | 0 |
| `0x20cb8f872ae894f7c9e32e621c186e5afce82fd0:0x0000000000000000000000000000000000000000000000000000000000000001` | unknown | 9 | 880556 | 0 |
| `0x22aee3699b6a0fed71490c103bd4e5f3309891d5:0x0000000000000000000000000000000000000000000000000000000000000006` | unknown | 9 | 979811 | 0 |
| `0x57135dd4b832645955422b28291302d697ea0900:0x0000000000000000000000000000000000000000000000000000000000000006` | unknown | 8 | 671150 | 0 |
| `0x0f12cb1f3e37375af09a248ca97c4a3eedf2f494:0x0000000000000000000000000000000000000000000000000000000000000000` | unknown | 6 | 637971 | 0 |
| `0x498581ff718922c3f8e6a244956af099b2652b2b:0x934ff9aed835597de93bbfd6474d60d33c1e72493b62f8e1615cfda0908eeaef` | unknown | 6 | 637971 | 0 |
| `0x4c9f68e780523feb4c9bb1aad2e5cc3b6476892b:0x0000000000000000000000000000000000000000000000000000000000000000` | unknown | 6 | 637971 | 0 |

## Warning Summary

- Data covers partial source transaction set: 25 of 436 transactions (5.734%).
- 25 of 25 analyzed txs: Provider support for debug_traceTransaction and JavaScript tracers varies
- 25 of 25 analyzed txs: read coverage is incomplete
- 25 of 25 analyzed txs: tracer records SLOAD/SSTORE storage observations only
- 2 of 25 analyzed txs: tracer reported faults
- Provider trace completeness varies; verify tracer support before making claims
- block 38014901: collection truncated to 25 of 436 transactions by --max-transactions
- gas-weighted scheduling is theoretical, not measured throughput

Full per-transaction warnings are preserved in `dossier.json`.

## What This Proves

This dossier shows deterministic access-contention structure, hot-state concentration, gas-weighted theoretical scheduling bounds where gas is available, and worker-count sensitivity for the supplied trace pack.

## What This Does Not Prove

It is not production TPS, not Ggas/s, does not execute/replay full EVM state transitions, and is not proof that observed access hints are complete Ethereum access lists.
