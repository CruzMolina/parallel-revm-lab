# Correctness

## Main Invariant

For a fixed initial state and canonical transaction list:

```text
hash(sequential(txs)) == hash(access_list(txs)) == hash(optimistic(txs))
```

Unit and property tests also compare full final `State` values, not only hashes.

## State Hash Invariant

State hashes are stable across process runs because state is stored in deterministic maps and hashed through an explicit byte encoding. Tests verify that identical seeded states produce identical hashes and that deltas change hashes.

## Conflict Model

Two transactions conflict if either writes a key the other reads or writes. Read/read overlap is allowed. Access-list waves are pairwise independent under this model before they execute in parallel. The scheduler also preserves canonical order by preventing later transactions from entering a wave when they conflict with an earlier transaction already deferred out of that wave.

## Trace-Derived Conflict Graph

For normalized block traces, the analyzer builds directed dependencies from lower `tx_index` to higher `tx_index` for every conflict pair. The wave invariant is:

```text
for every dependency earlier -> later:
    wave(earlier) < wave(later)
```

Additional trace analyzer invariants:

- waves partition every transaction index exactly once
- wave assignment is deterministic for the same normalized trace
- hot contract and hot slot rankings break ties by key
- duplicate `tx_index` values are rejected
- malformed fixture addresses, storage keys, and transaction hashes are rejected
- incomplete read information emits warnings and marks conflict counts as lower bounds

## Re-Execution Strategy

Optimistic execution records every value read during speculative execution. Canonical validation compares those values against committed state. A changed read triggers deterministic re-execution against current state.

## Edge Cases

Covered edge cases include:

- zero transactions
- one transaction
- high-contention `c95` workloads
- deterministic insufficient-balance behavior
- no-op transactions
- storage writes that optionally skip prior reads
- `vm_steps` synthetic CPU work that does not affect state semantics
- duplicate trace transaction indices
- invalid fixture addresses, storage keys, and transaction hashes
- trace-pack schema validation, manifest roundtrip, deterministic access normalization, and missing-gas fallback
- gas-weighted critical path and worker simulation on a known DAG
- incomplete trace read information
- deterministic fixture wave shape and report hash
- Geth struct-log fixture parsing for `SLOAD`/`SSTORE`
- revm inspector bytecode fixtures for hot-slot and independent-slot behavior, including trace-pack conversion

## Property Testing

`crates/executor` uses `proptest` to generate small seeded workloads across all workload kinds and conflict levels. Each case asserts that access-list and optimistic execution produce the same final hash and the same full final `State` as sequential execution.

## What The Tests Do Not Prove

- They do not prove optimal scheduling.
- They do not prove EVM semantic equivalence.
- They do not prove production performance.
- They do not prove the non-cryptographic state hash is collision resistant.
- Trace analyzer tests do not prove provider RPC traces are complete.
- The Geth parser tests do not prove every Geth trace shape.
- The revm smoke tests do not prove full block replay or account/code/balance extraction.
- Worker simulation tests prove deterministic scheduling over the modeled dependency graph, not real executor performance.

They do prove the central lab invariant for the implemented synthetic model: parallel modes are sequentially equivalent across fixed, randomized, and high-contention workloads.
