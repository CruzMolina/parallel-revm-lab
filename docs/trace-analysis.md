# Trace Analysis

The trace analyzer turns normalized or offline EVM-like access observations into a deterministic contention report. The tested ingestion paths are normalized fixtures and a tiny Geth-style struct-log fixture. Live RPC mode is intentionally documented as provider-dependent future work.

## Normalized Model

`crates/trace-model` defines:

- `ChainKind` and `BlockRef`
- `TxIndex`, `TxHash`, `Address`, and `StorageKey`
- `TraceAccessKey` for account, balance, nonce, code, and storage slot access
- `TraceAccessKind` for read, write, and read/write observations
- `TxTrace` and `BlockAccessTrace`
- `TraceParseWarning`

The parser canonicalizes address/storage strings, sorts access lists and warnings, sorts transactions by `tx_index`, and rejects duplicate transaction indices. Validated normalized fixtures must use:

- addresses: `0x` plus 40 hex characters
- storage keys: `0x` plus 64 hex characters
- transaction hashes: `0x` plus 64 hex characters

Validation errors include `tx_index` context where the malformed value appears.

## Geth Struct-Log Fixture

`analyze-trace --format geth-struct-logs` supports the committed tiny fixture shape in `fixtures/geth-mini-struct-logs.json`.

The parser:

- reads transaction-level `from`, `to`, `tx_index`, and `tx_hash`
- scans `structLogs`
- records `SLOAD` as a storage read
- records `SSTORE` as a storage write
- derives the storage slot from the top stack value
- marks read information incomplete because account, balance, nonce, code, and non-storage observations are not represented

This is intentionally not a complete Geth trace implementation. It is one concrete offline format that proves the ingestion path without live RPC or fabricated chain data.

## Conflict Assumptions

Two transactions conflict if:

- both write the same key
- an earlier write overlaps a later read
- an earlier read overlaps a later write

Read/read overlap is allowed. Dependencies preserve canonical transaction order by pointing from the earlier conflicting transaction to the later one.

If a trace marks reads incomplete, the report is labeled `declared-read-write-lower-bound`. Missing reads can hide conflicts, so the report is not conservative in that case.

## Provider And RPC Caveats

Different Ethereum providers expose different debug and trace APIs. Even when a trace endpoint exists, read information can be incomplete or shaped differently across clients. For that reason:

- CI uses normalized fixture and Geth struct-log fixture mode only.
- RPC URLs are never printed.
- `--rpc-url` takes precedence over `BASE_RPC_URL` and `ETH_RPC_URL`.
- Live RPC normalization should be implemented provider by provider with fixtures and tests.

## Why Fixture Mode Exists

Fixture mode gives reviewers deterministic artifacts with no secrets, no provider dependency, and no fabricated live-chain claim. The committed normalized fixture is synthetic and Base-shaped; the committed Geth fixture is sanitized and intentionally tiny.
