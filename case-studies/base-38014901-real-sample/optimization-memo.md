# Optimization Memo

## What The Trace Shows

The first 25 transactions of Base block `38014901` covered 3,584,277 gas and 680 observed storage accesses across 62 contracts and 226 storage slots. The sample has high observed overlap: 22 of 25 transactions share at least one observed key with another sampled transaction.

## What Serialized The Schedule

Only one observed write-related conflict pair created a dependency edge under this repository's model. The serializing pair touched `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789`, with the top conflict slots contributing one conflict each. The tx-count critical path is 2, and the gas-weighted critical path is 563,160.

## Where Parallelism Was Available

The deterministic waves are wide: 2 waves total with max width 24. The theoretical ceiling is 12.500x by transaction count and 6.365x by gas. Simulated workers reach 6.168x at 16 workers, with 61.45% idle time, which means the sample is mostly limited by gas-weighted task balance and the remaining dependency edge at high worker counts.

## What Hot Contracts And Slots Dominated

The top conflict-contributing contract is `0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789`. The top two conflict slots on that contract each contributed one conflict. Several other contracts and slots were touched by many transactions but contributed no write-related conflicts in this sample, so they are better read-sharing/cache candidates than serial bottlenecks.

## What I Would Optimize Next

- Better access-set prediction for recurring hot storage keys before execution.
- Hot-slot aware scheduling that separates real write conflicts from read/read overlap.
- Speculative execution with targeted re-execution for the small set of write-conflicting keys.
- Read/write set caching for high-overlap, read-compatible storage keys.
- State-prefetch hints for the repeated hot contracts and slots.
- DB read coalescing for shared read-heavy keys.
- Worker balancing by gas-weighted duration, because the 16-worker schedule is close to the gas critical path but still has high idle time.
- Conflict-aware batching that isolates the `0x5ff137d4...` dependency while keeping the wide independent wave saturated.

## What This Analysis Cannot Prove

This is not a production execution client, not full block replay, not a complete EVM dependency extractor, and not a TPS or Ggas/s benchmark. The sample covers only 25 of 436 block transactions, and the tracer observes storage opcodes only.
