# Trace Analysis

The trace analyzer turns normalized or offline EVM-like access observations into deterministic contention reports. The tested ingestion paths are normalized fixtures, a tiny Geth-style struct-log fixture, and compact trace packs. Live RPC collection is optional and provider-dependent.

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

## Trace-Pack Schema

A trace pack is a compact directory:

```text
trace-packs/<name>/
  manifest.json
  blocks/
    <block-number>.json
```

The manifest records schema version, chain, source, provenance, start/end block, tool version, tracer kind, notes, and warnings. Each block file records block identity, optional gas totals, ordered transactions, optional receipt gas/status, and compact access observations.

Validated fields:

- addresses: `0x` plus 40 hex characters
- storage slots: `0x` plus 64 hex characters
- tx hashes: `0x` plus 64 hex characters
- block files must match the manifest chain/range
- adjacent block hashes must chain through `parent_hash` when both hashes are present
- complete block gas must equal the sum of transaction `gas_used` values
- duplicate transactions and duplicate exact accesses are normalized deterministically

Missing gas is allowed. Dossier reports then mark gas-weighted metrics unavailable and fall back to one duration unit per tx for worker simulation.

## Gas-Weighted Scheduling

`analyze-trace-pack` computes tx-count metrics and gas-weighted metrics. Tx-count metrics include conflict pairs, deterministic waves, critical path length, and `tx_count / critical_path`. Gas-weighted metrics include weighted conflict percentage, gas critical path, and `total_gas / gas_critical_path` when every included tx has gas and the block total reconciles with transaction gas.

Worker simulation uses deterministic list scheduling over the dependency graph. It uses receipt `gas_used` as task duration when available, otherwise one unit per transaction. This is a theoretical model, not measured Ggas/s.

## Observed Access Hints

`recommend-access-lists` emits observed contracts and storage keys per transaction plus the keys responsible for the most observed conflicts. The output is labeled as access hints, not complete Ethereum access lists. Dynamic accesses and incomplete traces can be missing.

## Conflict Assumptions

Two transactions conflict if:

- both write the same key
- an earlier write overlaps a later read
- an earlier read overlaps a later write

Read/read overlap is allowed. Dependencies preserve canonical transaction order by pointing from the earlier conflicting transaction to the later one.

If a trace marks reads incomplete, the report is labeled `declared-read-write-lower-bound`. Missing reads can hide conflicts, so the report is not conservative in that case.

## Provider And RPC Caveats

Different Ethereum providers expose different debug and trace APIs. Even when a trace endpoint exists, read information can be incomplete or shaped differently across clients. For that reason:

- CI uses normalized fixture, Geth struct-log fixture, and committed demo trace-pack mode only.
- RPC URLs are never printed.
- `--rpc-url` takes precedence over `BASE_RPC_URL` and `ETH_RPC_URL`.
- `collect-block-range --resume` resumes from validated per-block trace-pack files already written under `blocks/`.
- Live RPC normalization should be implemented provider by provider with fixtures and tests.

## Why Fixture Mode Exists

Fixture mode gives reviewers deterministic artifacts with no secrets, no provider dependency, and no fabricated live-chain claim. The committed normalized fixture is synthetic and Base-shaped; the committed Geth fixture is sanitized and intentionally tiny.
