# Parallelism Report: synthetic-base-shaped-fixture block 900000000

## Summary

- Transactions: 12
- Conflict pairs: 7 (10.606%)
- Waves: 3
- Max wave width: 7
- Critical path length: 3
- Theoretical parallelism ceiling: 4.000x

## Hot Contracts

| contract | accesses |
| --- | ---: |
| `0x1111111111111111111111111111111111111111` | 5 |
| `0x2222222222222222222222222222222222222222` | 3 |
| `0x3333333333333333333333333333333333333333` | 2 |
| `0x4444444444444444444444444444444444444444` | 1 |
| `0x5555555555555555555555555555555555555555` | 1 |

## Hot Storage Slots

| slot | accesses |
| --- | ---: |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000001` | 3 |
| `0x2222222222222222222222222222222222222222:0x00000000000000000000000000000000000000000000000000000000deadbeef` | 3 |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000000` | 2 |
| `0x3333333333333333333333333333333333333333:0x0000000000000000000000000000000000000000000000000000000000000007` | 1 |
| `0x4444444444444444444444444444444444444444:0x0000000000000000000000000000000000000000000000000000000000000042` | 1 |

## Waves

| tx | wave | degree | reads | writes |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 0 | 1 | 1 | 1 |
| 1 | 0 | 2 | 1 | 1 |
| 2 | 0 | 2 | 1 | 1 |
| 3 | 1 | 2 | 1 | 1 |
| 4 | 0 | 0 | 1 | 1 |
| 5 | 1 | 1 | 1 | 1 |
| 6 | 2 | 2 | 1 | 1 |
| 7 | 0 | 0 | 0 | 1 |
| 8 | 1 | 2 | 1 | 0 |
| 9 | 2 | 2 | 0 | 1 |
| 10 | 0 | 0 | 0 | 1 |
| 11 | 0 | 0 | 1 | 1 |

## Warnings

- fixture is synthetic and is not claimed to be real Base chain data
- tx_index 10: analysis is a lower bound: read set is marked incomplete
- tx_index 10: trace marks read information as incomplete

## Caveats

This report studies access-set contention and deterministic scheduling structure. It is not a production throughput or gas benchmark. If a trace marks reads incomplete, conflict counts are lower bounds.
