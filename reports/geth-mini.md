# Parallelism Report: geth-struct-logs-fixture block 1

## Summary

- Transactions: 3
- Conflict pairs: 1 (33.333%)
- Waves: 2
- Max wave width: 2
- Critical path length: 2
- Theoretical parallelism ceiling: 1.500x

## Hot Contracts

| contract | accesses |
| --- | ---: |
| `0x1111111111111111111111111111111111111111` | 4 |

## Hot Storage Slots

| slot | accesses |
| --- | ---: |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000000` | 3 |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000001` | 1 |

## Waves

| tx | wave | degree | reads | writes |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 0 | 1 | 1 | 1 |
| 1 | 0 | 0 | 0 | 1 |
| 2 | 1 | 1 | 0 | 1 |

## Warnings

- fixture is sanitized and intentionally tiny; it is not a real mainnet transaction trace
- geth struct-log parser captures SLOAD/SSTORE storage accesses only; account, balance, nonce, and code reads are not represented
- tx_index 0: analysis is a lower bound: read set is marked incomplete
- tx_index 0: geth struct-log storage parser marks reads incomplete outside SLOAD
- tx_index 0: trace marks read information as incomplete
- tx_index 1: analysis is a lower bound: read set is marked incomplete
- tx_index 1: geth struct-log storage parser marks reads incomplete outside SLOAD
- tx_index 1: trace marks read information as incomplete
- tx_index 2: analysis is a lower bound: read set is marked incomplete
- tx_index 2: geth struct-log storage parser marks reads incomplete outside SLOAD
- tx_index 2: trace marks read information as incomplete

## Caveats

This report studies access-set contention and deterministic scheduling structure. It is not a production throughput or gas benchmark. If a trace marks reads incomplete, conflict counts are lower bounds.
