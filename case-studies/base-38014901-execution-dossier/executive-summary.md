# Contention Dossier: base blocks 38014901-38014901

**Provenance:** `user-collected RPC trace pack`

**Tracer:** `geth-js-storage`

## Summary

- Blocks: 1
- Transactions: 436
- Source transactions covered: 436 of 436 (100.000%)
- Accesses: 27034
- Conflict pairs: 5052 (5.327%)
- Overlapping transactions: 403 (92.431%)
- Overlap is broader than serialization: read-compatible overlap can be high even when write-dependent conflict pairs and waves stay low.
- Waves: 60
- Max wave width: 106
- Critical path by tx count: 60
- Theoretical ceiling by tx: 7.267x
- Total gas covered: 71655982
- Gas-weighted critical path: 16951990
- Theoretical ceiling by gas: 4.227x

## Worker Simulation

| workers | makespan | speedup | idle | interpretation |
| ---: | ---: | ---: | ---: | --- |
| 1 | 71655982 | 1.000x | 0.00% | worker-bound: workers stay mostly occupied under observed dependencies |
| 2 | 36389121 | 1.969x | 1.54% | worker-bound: workers stay mostly occupied under observed dependencies |
| 4 | 19359165 | 3.701x | 7.47% | mixed dependency/worker-bound: dependencies and idle capacity both matter |
| 8 | 17381589 | 4.123x | 48.47% | mixed dependency/worker-bound: dependencies and idle capacity both matter |
| 16 | 16953037 | 4.227x | 73.58% | mixed dependency/worker-bound: dependencies and idle capacity both matter |

## Scheduler Ablation

All strategies preserve observed dependencies; they only change deterministic ready-queue priority.

| strategy | workers | makespan | speedup vs canonical 1-worker | idle | ready wait | critical-path bound | improvement vs canonical | deps preserved |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `canonical` | 1 | 71655982 | 1.000x | 0.00% | 4022895361 | 16951990 | 0.00% | true |
| `canonical` | 2 | 36389121 | 1.969x | 1.54% | 1946047757 | 16951990 | 0.00% | true |
| `canonical` | 4 | 19359165 | 3.701x | 7.47% | 542752310 | 16951990 | 0.00% | true |
| `canonical` | 8 | 17381589 | 4.123x | 48.47% | 188262374 | 16951990 | 0.00% | true |
| `canonical` | 16 | 16953037 | 4.227x | 73.58% | 61373991 | 16951990 | 0.00% | true |
| `gas_lpt` | 1 | 71655982 | 1.000x | 0.00% | 3066311505 | 16951990 | 0.00% | true |
| `gas_lpt` | 2 | 36994895 | 1.937x | 3.15% | 1380746646 | 16951990 | -1.66% | true |
| `gas_lpt` | 4 | 22694599 | 3.157x | 21.06% | 512902291 | 16951990 | -17.23% | true |
| `gas_lpt` | 8 | 18718873 | 3.828x | 52.15% | 209800774 | 16951990 | -7.69% | true |
| `gas_lpt` | 16 | 17564316 | 4.080x | 74.50% | 81554763 | 16951990 | -3.61% | true |
| `critical_path` | 1 | 71655982 | 1.000x | 0.00% | 7619968599 | 16951990 | 0.00% | true |
| `critical_path` | 2 | 35829941 | 2.000x | 0.01% | 3792087940 | 16951990 | 1.54% | true |
| `critical_path` | 4 | 17914903 | 4.000x | 0.01% | 1520695740 | 16951990 | 7.46% | true |
| `critical_path` | 8 | 16951990 | 4.227x | 47.16% | 295153026 | 16951990 | 2.47% | true |
| `critical_path` | 16 | 16951990 | 4.227x | 73.58% | 100727251 | 16951990 | 0.01% | true |

## Worst Serializing Transactions

| block | tx | wave | conflicts | duration | tx hash |
| ---: | ---: | ---: | ---: | ---: | --- |
| 38014901 | 114 | 9 | 84 | 432891 | `0x02ca888cf2fd6ab09ad0b06ad2b1d89d526172e2e201a47adc47ca0b8bd95fd4` |
| 38014901 | 95 | 5 | 81 | 132632 | `0xe4ea119341cf214d93fd55e3c23b3ef944aebf51e120e0a1d00e640152475cf4` |
| 38014901 | 149 | 14 | 81 | 123940 | `0x05adc1a778ed2f5c583ecedca8adf74e998a4a100998407803d29a210f01d436` |
| 38014901 | 315 | 44 | 77 | 594524 | `0x5ad07895f0f6d7d72fd769519eb195219ff2091f35c122e7e2ec3bfb29552f84` |
| 38014901 | 318 | 45 | 77 | 503628 | `0xb28824f578fc037a2e3c290e1f8e05f85735d5c99514bcfa2bcb19e0f477e22a` |
| 38014901 | 224 | 21 | 77 | 501756 | `0xac969a85a6dbe5bcfe765864c3b17df7629e0439ce0ec14f31f815f26789eb9b` |
| 38014901 | 96 | 5 | 77 | 500496 | `0x8de117fb4a0e2710a3f2465ec84f182fafad6f8cb64e13d2b3d900a5b4e18526` |
| 38014901 | 162 | 13 | 77 | 495915 | `0x278292e4812b721a3fde29ea42d578c2048d95ea3f399bc00722ef1f60e36235` |
| 38014901 | 225 | 22 | 77 | 491435 | `0x6e592a5c2435b2f178393cc7059d2447b52a2a828412015031bd80125b43dcb6` |
| 38014901 | 97 | 6 | 77 | 490425 | `0x9e6e58bbdf249546dc3369ca1ae37e74c38c4a7cb333c4c2a6eebaa1663d6176` |

## Hot Contracts

| contract | txs | unique slots | gas covered | conflict contribution |
| --- | ---: | ---: | ---: | ---: |
| `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913` | 177 | 208 | 34490590 | 2543 |
| `0x4200000000000000000000000000000000000006` | 153 | 117 | 34105958 | 2373 |
| `0xb2cc224c1c9fee385f8ad6a55b4d94e92359dc59` | 47 | 54 | 12020811 | 1704 |
| `0x827922686190790b37229fd06084350e74485b72` | 33 | 368 | 15433386 | 606 |
| `0x72ab388e2e2f6facef59e3c3fa2c4e29011c2d38` | 49 | 14 | 6310331 | 481 |
| `0x20cb8f872ae894f7c9e32e621c186e5afce82fd0` | 61 | 7 | 8149938 | 360 |
| `0xd0b53d9277642d899df5c87a3966a349a798f224` | 60 | 6 | 11618931 | 354 |
| `0x70acdf2ad0bf2402c957154f944c19ef4e1cbae1` | 37 | 34 | 6010520 | 319 |
| `0x22aee3699b6a0fed71490c103bd4e5f3309891d5` | 54 | 52 | 8636780 | 245 |
| `0x4e962bb3889bf030368f56810a9c96b83cb3e778` | 7 | 54 | 2788929 | 138 |

## Hot Storage Slots

| slot | txs | gas covered | conflict contribution |
| --- | ---: | ---: | ---: |
| `0x4200000000000000000000000000000000000006:0x0d52ad225b9f8da090dc37c741705dabc30f648dce00d7b0cab66994a1261ea6` | 60 | 7037020 | 1704 |
| `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913:0xb11ac13f71492d75461bf3c1f12c195cbe42b046a859ec6a34848efc0462deda` | 59 | 6907579 | 1645 |
| `0xb2cc224c1c9fee385f8ad6a55b4d94e92359dc59:0x0000000000000000000000000000000000000000000000000000000000000006` | 47 | 12020811 | 675 |
| `0x827922686190790b37229fd06084350e74485b72:0x0000000000000000000000000000000000000000000000000000000000000002` | 33 | 15433386 | 375 |
| `0xd0b53d9277642d899df5c87a3966a349a798f224:0x0000000000000000000000000000000000000000000000000000000000000000` | 60 | 11618931 | 339 |
| `0x4200000000000000000000000000000000000006:0x7fa89ed191104b59893c54aa61c35c4b88ae911f1ac7cab067c891ab0601acac` | 27 | 6928640 | 306 |
| `0x72ab388e2e2f6facef59e3c3fa2c4e29011c2d38:0x0000000000000000000000000000000000000000000000000000000000000000` | 49 | 6310331 | 230 |
| `0x72ab388e2e2f6facef59e3c3fa2c4e29011c2d38:0x0000000000000000000000000000000000000000000000000000000000000001` | 49 | 6310331 | 230 |
| `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913:0x6b44a3e0842ecc28ca534a294955223b94a5209f3671031126ec16ea6e217dab` | 22 | 6732792 | 221 |
| `0x22aee3699b6a0fed71490c103bd4e5f3309891d5:0x0000000000000000000000000000000000000000000000000000000000000006` | 54 | 8636780 | 206 |

## Warning Summary

- 436 of 436 analyzed txs: Geth JavaScript tracer records SLOAD/SSTORE storage observations only.
- 436 of 436 analyzed txs: Provider support for debug_traceTransaction and JavaScript tracers varies.
- 31 of 436 analyzed txs: debug tracer reported 1 fault(s)
- 25 of 436 analyzed txs: debug tracer reported 2 fault(s)
- 3 of 436 analyzed txs: debug tracer reported 3 fault(s)
- 4 of 436 analyzed txs: debug tracer reported 4 fault(s)
- 1 of 436 analyzed txs: debug tracer reported 5 fault(s)
- 1 of 436 analyzed txs: debug tracer reported 7 fault(s)
- 1 of 436 analyzed txs: debug tracer reported 8 fault(s)
- 436 of 436 analyzed txs: trace marks read information as incomplete
- Provider trace completeness varies; verify tracer support before making claims
- block 38014901: trace pack access observations may be incomplete; gas-weighted scheduling is theoretical

Full per-transaction warnings are preserved in `dossier.json`.

## What This Proves

This dossier shows deterministic access-contention structure, hot-state concentration, gas-weighted theoretical scheduling bounds where gas is available, and worker-count sensitivity for the supplied trace pack.

## What This Does Not Prove

It is not production TPS, not Ggas/s, not full block replay, and not proof that observed access hints are complete Ethereum access lists.
