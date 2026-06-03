# Design

## State Model

The internal model has accounts, balances, nonces, contract storage slots, access keys, transactions, read/write sets, deltas, execution outcomes, and stable state hashes. It is intentionally smaller than EVM execution, but shaped around the same scheduler problem: transactions read and write overlapping state keys.

State storage uses deterministic maps:

- `BTreeMap<AccountId, Account>` for accounts.
- `BTreeMap<(ContractId, SlotId), i128>` for storage.
- `BTreeSet<AccessKey>` for access sets.

The state hash is an explicit FNV-1a 64-bit hash over canonical bytes. It avoids `DefaultHasher` because persistent comparisons must not depend on implementation-specific hash seeding or map iteration order.

## Trace Model

`crates/trace-model` represents normalized EVM/block access observations independently from live RPC. It models:

- chain and block identity
- canonical transaction index and hash
- account, balance, nonce, code, and storage access keys
- read, write, and read/write access kinds
- per-transaction and block-level warnings

The fixture parser sorts transactions by `tx_index`, canonicalizes addresses and storage keys, rejects duplicate transaction indices, validates EVM hex-shaped addresses/storage keys/tx hashes, and marks incomplete read information explicitly. All user-visible output depends on sorted vectors, `BTreeMap`, or `BTreeSet`.

The Geth struct-log parser supports one committed offline fixture shape. It scans `structLogs`, records `SLOAD` as storage reads, records `SSTORE` as storage writes, and derives the slot from the top stack value. It intentionally marks broader read coverage incomplete.

Trace packs extend the normalized model with a manifest, per-block files, gas-used metadata, provenance, and compact per-op observations. The schema is deterministic: block files are loaded by manifest range, transactions sort by `tx_index`, exact duplicate accesses are deduplicated, and all persistent output uses sorted maps or vectors.

## Trace Analyzer

`crates/analyzer` consumes a `BlockAccessTrace` and builds pairwise conflicts. A later transaction depends on an earlier transaction when their access sets overlap through write/write, earlier write/later read, or earlier read/later write. Read/read overlap is allowed.

The wave builder is deterministic: it repeatedly selects all remaining transactions whose dependencies are already assigned. This yields a lower-bound schedule shape for incomplete-read traces and an exact schedule for complete declared access traces under the implemented conflict model.

## Transaction Semantics

- `Transfer`: reads sender balance, sender nonce, recipient balance; writes sender balance, sender nonce, recipient balance. Insufficient balance deterministically increments the sender nonce and leaves balances unchanged.
- `StorageAdd`: reads and writes one storage slot with saturating arithmetic.
- `StorageSet`: writes one slot and optionally reads the prior value.
- `SwapLike`: reads/writes two reserve slots and one account balance using a deterministic constant-product-like quote.
- `HotPool`: reads/writes one shared pool slot and one account balance to create contention.
- `Noop`: touches no state.

Each transaction can also carry `vm_steps`, an optional deterministic CPU loop that simulates interpreter work. It does not read or write state and does not change transaction semantics.

Invalid or insufficient operations return deterministic outcomes instead of panicking.

## Sequential Baseline

Sequential execution applies transactions in input order against mutable state. Every parallel executor is measured against this final state hash and, in tests, the final state itself.

## Access-List Wave Scheduler

The access-list scheduler greedily builds waves from pending transactions. A transaction can join the current wave only if it has no write/write, read/write, or write/read conflict with the aggregate wave access sets. It also cannot skip ahead of an earlier deferred transaction that conflicts with it; deferred conflicting transactions act as canonical-order barriers. Each wave executes in parallel against the same pre-wave snapshot. Deltas are committed in canonical transaction order.

This is conservative: it may create more waves than an optimal scheduler, but it keeps correctness easy to audit.

## Optimistic Scheduler

The optimistic executor speculates all candidate transactions in parallel against a snapshot. It records observed read values. During canonical validation, each transaction's observed reads are compared with the evolving committed state. If any read changed, the transaction is re-executed against current state and the fresh delta is committed.

This favors correctness over maximum speed. High contention can cause many re-executions.

## Deterministic Commit Model

Worker execution order never determines final state. Both parallel executors commit in canonical transaction order. JSON report fields are struct ordered, and state/access structures use deterministic ordering.

## Metrics

Reports avoid a single ambiguous conflict counter:

- `declared_conflict_pairs`: pairwise conflicts implied by declared access sets.
- `scheduler_deferrals`: access-list deferral decisions while forming waves. This can exceed `declared_conflict_pairs` because the same transaction can be deferred across multiple waves.
- `validation_failures`: optimistic read-validation failures.
- `reexecuted_txs`: optimistic transactions re-executed after validation failure.
- `wave_count` and `max_wave_width`: access-list wave shape, or optimistic batch shape.

Trace-analysis reports add:

- `conflict_pair_count` and `conflict_percentage`
- `critical_path_length`
- `theoretical_parallelism_ceiling`
- hot contract and hot storage slot rankings
- per-transaction read/write counts, conflict degree, and wave
- a stable report hash over canonical JSON bytes
- trace-pack dossiers with gas-weighted critical paths, worker simulation, hot-state concentration, and CSV sidecars

Synthetic throughput metrics and trace-analysis parallelism metrics are intentionally separate.

## revm Smoke Bridge

`crates/revm-smoke` is intentionally small. It uses `revm 40.0.3` with `default-features = false` and `std` enabled, executes tiny bytecode fixtures against `CacheDB<EmptyDB>`, records `SLOAD` and `SSTORE` with a revm inspector, and converts those observations into the normalized trace model.

This proves the analyzer can ingest observations from real EVM bytecode execution. The smoke crate can also emit a tiny trace pack and dossier. It does not attempt general block replay, full account/code/balance inspector coverage, or provider RPC normalization.

## Concurrency Choice

The project uses Rayon `ThreadPoolBuilder` rather than custom queues or shared mutable concurrent state. Rayon provides reliable parallel iteration while the model keeps mutation in a deterministic commit phase. Because no custom concurrency primitive is introduced, loom tests are not included.

## Tradeoffs

- Synthetic access lists make scheduler behavior inspectable, but do not prove real EVM access-list extraction.
- The optimistic executor currently speculates from one snapshot per run, which is simple and honest but not throughput-optimal for very long batches.
- `vm_steps` is only deterministic CPU work for exploring scheduling overhead versus execution cost.
- The state hash is stable and deterministic, but not a cryptographic commitment.
- Incomplete trace reads make analyzer conflict counts lower bounds.
- Live RPC tracing varies by provider and is not part of CI.
- The current Geth parser and revm inspector are storage-access focused.
- Trace-pack worker simulation is theoretical scheduling, not measured execution throughput.
