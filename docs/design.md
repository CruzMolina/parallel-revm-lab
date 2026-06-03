# Design

## State Model

The internal model has accounts, balances, nonces, contract storage slots, access keys, transactions, read/write sets, deltas, execution outcomes, and stable state hashes. It is intentionally smaller than EVM execution, but shaped around the same scheduler problem: transactions read and write overlapping state keys.

State storage uses deterministic maps:

- `BTreeMap<AccountId, Account>` for accounts.
- `BTreeMap<(ContractId, SlotId), i128>` for storage.
- `BTreeSet<AccessKey>` for access sets.

The state hash is an explicit FNV-1a 64-bit hash over canonical bytes. It avoids `DefaultHasher` because persistent comparisons must not depend on implementation-specific hash seeding or map iteration order.

## Transaction Semantics

- `Transfer`: reads sender balance, sender nonce, recipient balance; writes sender balance, sender nonce, recipient balance. Insufficient balance deterministically increments the sender nonce and leaves balances unchanged.
- `StorageAdd`: reads and writes one storage slot with saturating arithmetic.
- `StorageSet`: writes one slot and optionally reads the prior value.
- `SwapLike`: reads/writes two reserve slots and one account balance using a deterministic constant-product-like quote.
- `HotPool`: reads/writes one shared pool slot and one account balance to create contention.
- `Noop`: touches no state.

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

## Concurrency Choice

The project uses Rayon `ThreadPoolBuilder` rather than custom queues or shared mutable concurrent state. Rayon provides reliable parallel iteration while the model keeps mutation in a deterministic commit phase. Because no custom concurrency primitive is introduced, loom tests are not included.

## Tradeoffs

- Synthetic access lists make scheduler behavior inspectable, but do not prove real EVM access-list extraction.
- The optimistic executor currently speculates from one snapshot per run, which is simple and honest but not throughput-optimal for very long batches.
- The state hash is stable and deterministic, but not a cryptographic commitment.
