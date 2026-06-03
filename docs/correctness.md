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

Two transactions conflict if either writes a key the other reads or writes. Read/read overlap is allowed. Access-list waves are pairwise independent under this model before they execute in parallel.

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

## Property Testing

`crates/executor` uses `proptest` to generate small seeded workloads across all workload kinds and conflict levels. Each case asserts that access-list and optimistic execution produce the same final hash as sequential execution.

## What The Tests Do Not Prove

- They do not prove optimal scheduling.
- They do not prove EVM semantic equivalence.
- They do not prove production performance.
- They do not prove the non-cryptographic state hash is collision resistant.

They do prove the central lab invariant for the implemented synthetic model: parallel modes are sequentially equivalent across fixed, randomized, and high-contention workloads.
