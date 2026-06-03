# Trace Analysis

The trace analyzer turns normalized EVM-like access observations into a deterministic contention report. Fixture mode is the stable path; live RPC mode is intentionally documented as provider-dependent future work.

## Normalized Model

`crates/trace-model` defines:

- `ChainKind` and `BlockRef`
- `TxIndex`, `TxHash`, `Address`, and `StorageKey`
- `TraceAccessKey` for account, balance, nonce, code, and storage slot access
- `TraceAccessKind` for read, write, and read/write observations
- `TxTrace` and `BlockAccessTrace`
- `TraceParseWarning`

The parser canonicalizes address/storage strings, sorts access lists and warnings, sorts transactions by `tx_index`, and rejects duplicate transaction indices.

## Conflict Assumptions

Two transactions conflict if:

- both write the same key
- an earlier write overlaps a later read
- an earlier read overlaps a later write

Read/read overlap is allowed. Dependencies preserve canonical transaction order by pointing from the earlier conflicting transaction to the later one.

If a trace marks reads incomplete, the report is labeled `declared-read-write-lower-bound`. Missing reads can hide conflicts, so the report is not conservative in that case.

## Provider And RPC Caveats

Different Ethereum providers expose different debug and trace APIs. Even when a trace endpoint exists, read information can be incomplete or shaped differently across clients. For that reason:

- CI uses fixture mode only.
- RPC URLs are never printed.
- `--rpc-url` takes precedence over `BASE_RPC_URL` and `ETH_RPC_URL`.
- Live RPC normalization should be implemented provider by provider with fixtures and tests.

## Why Fixture Mode Exists

Fixture mode gives reviewers a deterministic artifact with no secrets, no provider dependency, and no fabricated live-chain claim. The committed fixture is synthetic and Base-shaped, with enough hot and independent storage access to exercise conflict detection and wave construction.
