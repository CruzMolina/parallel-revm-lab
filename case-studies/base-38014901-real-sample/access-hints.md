# Observed Access Hints: base

**Provenance:** `user-collected RPC trace pack`

These are observed access hints, not complete production Ethereum access lists. Dynamic access and incomplete trace caveats apply.

## Candidate Conflict Keys

| key | conflict contribution |
| --- | ---: |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789:0x0000000000000000000000000000000000000000000000000000000000000002` | 1 |
| `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789:0x33a26eb216320a51847a061c8f256bd98ae96e1a1d67d2bbc4cd8c61adad2df0` | 1 |

## Scheduling-Helpful Transactions

| block | tx | conflicts | tx hash |
| ---: | ---: | ---: | --- |
| 38014901 | 23 | 1 | `0x009cd9a5c35c851ce4e38e94b7b1ed70aa2643ff040a7e72ddd4701ccf29da8d` |
| 38014901 | 24 | 1 | `0x0836dca8f01c33b320e570ea8a47687ca0a5f88a57af44d9cef3d7c9f30c9515` |

## Per-Transaction Observations

| block | tx | contracts | storage keys | warning |
| ---: | ---: | ---: | ---: | --- |
| 38014901 | 0 | 1 | 8 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 1 | 4 | 14 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 2 | 6 | 6 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 3 | 6 | 6 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 4 | 8 | 10 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 5 | 8 | 10 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 6 | 7 | 26 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 7 | 11 | 47 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 8 | 15 | 32 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 9 | 21 | 45 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 10 | 6 | 6 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 11 | 6 | 6 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 12 | 6 | 6 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 13 | 8 | 10 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 14 | 8 | 10 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 15 | 8 | 10 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 16 | 3 | 5 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 17 | 3 | 5 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 18 | 3 | 5 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 19 | 3 | 25 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 20 | 5 | 7 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 21 | 5 | 7 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 22 | 5 | 7 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 23 | 5 | 16 | observed access hints only; dynamic or unobserved accesses may be missing |
| 38014901 | 24 | 6 | 35 | observed access hints only; dynamic or unobserved accesses may be missing |

## Warnings

- observed access hints are not production-ready Ethereum access lists
- incomplete traces can miss dynamic storage keys and account/code/balance reads
