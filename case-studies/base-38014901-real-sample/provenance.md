# Provenance

- Chain: Base mainnet.
- Block: `38014901`.
- Block hash: `0x6a2428ff484a816874438551d68a9e036b6a5108b8dceab9fb10575534082881`.
- Collection date: June 3, 2026.
- RPC endpoint: user supplied; URL intentionally not recorded.
- Capability check: block fetch, receipts, custom JavaScript `debug_traceTransaction`, and struct-log tracing succeeded.
- Tracer: `tracers/geth-storage-access-tracer.js`.
- Trace pack: `trace-packs/base-38014901-real-sample`.
- Coverage: first 25 transactions of 436 block transactions, 5.734%.
- Compact output size: about 224 KB for the normalized trace pack.

## Limitations

- This is a partial sample, not a full block analysis.
- The tracer records storage opcode observations only: `SLOAD` and `SSTORE`.
- Account, code, balance, nonce, call/create, and non-storage read dependencies are not complete.
- The generated report marks reads incomplete and should be interpreted as a lower-bound contention model.
- Two transactions reported tracer faults while still producing compact observations.
- Worker simulation is theoretical deterministic scheduling over observed access keys, not measured execution throughput.

## Full-Range Status

A dry run over `38014901-38014910` succeeded and found 3,676 transactions total:

| block | transactions |
| ---: | ---: |
| 38014901 | 436 |
| 38014902 | 284 |
| 38014903 | 285 |
| 38014904 | 492 |
| 38014905 | 385 |
| 38014906 | 336 |
| 38014907 | 396 |
| 38014908 | 333 |
| 38014909 | 353 |
| 38014910 | 376 |

Full-range tracing was not attempted in this pass because it would require thousands of debug trace calls and likely produce much larger artifacts. The committed real artifact is the 25-transaction sample.
