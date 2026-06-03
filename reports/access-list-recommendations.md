# Observed Access Hints: synthetic-base-shaped

**Provenance:** `synthetic-base-shaped demo fixture (not real Base data)`

These are observed access hints, not complete production Ethereum access lists. Dynamic access and incomplete trace caveats apply.

## Candidate Conflict Keys

| key | conflict contribution |
| --- | ---: |
| `0x1111111111111111111111111111111111111111:0x0000000000000000000000000000000000000000000000000000000000000000` | 1 |
| `0x3333333333333333333333333333333333333333:0x0000000000000000000000000000000000000000000000000000000000000007` | 1 |

## Scheduling-Helpful Transactions

| block | tx | conflicts | tx hash |
| ---: | ---: | ---: | --- |
| 900000001 | 0 | 1 | `0x0000000000000000000000000000000000000000000000000000000000000100` |
| 900000001 | 2 | 1 | `0x0000000000000000000000000000000000000000000000000000000000000102` |
| 900000002 | 0 | 1 | `0x0000000000000000000000000000000000000000000000000000000000000200` |
| 900000002 | 2 | 1 | `0x0000000000000000000000000000000000000000000000000000000000000202` |

## Per-Transaction Observations

| block | tx | contracts | storage keys | warning |
| ---: | ---: | ---: | ---: | --- |
| 900000001 | 0 | 1 | 1 | observed access hints only; dynamic or unobserved accesses may be missing |
| 900000001 | 1 | 1 | 1 | observed access hints only; dynamic or unobserved accesses may be missing |
| 900000001 | 2 | 1 | 1 | observed access hints only; dynamic or unobserved accesses may be missing |
| 900000001 | 3 | 1 | 1 | observed access hints only; dynamic or unobserved accesses may be missing |
| 900000002 | 0 | 1 | 1 | observed access hints only; dynamic or unobserved accesses may be missing |
| 900000002 | 1 | 1 | 1 | observed access hints only; dynamic or unobserved accesses may be missing |
| 900000002 | 2 | 1 | 1 | observed access hints only; dynamic or unobserved accesses may be missing |

## Warnings

- observed access hints are not production-ready Ethereum access lists
- incomplete traces can miss dynamic storage keys and account/code/balance reads
