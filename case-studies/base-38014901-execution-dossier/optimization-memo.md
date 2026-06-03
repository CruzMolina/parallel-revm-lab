# Optimization Memo

## What The Trace Shows

Base block `38014901` has high observed access overlap but materially less write-dependent serialization. The dossier saw 403 of 436 transactions overlap with at least one other transaction by observed key, while pairwise conflicts were 5,052 of 94,830 possible pairs, or 5.327%.

The gas-weighted critical path was 16,951,990 gas units out of 71,655,982 total gas covered, leaving a theoretical gas ceiling of 4.227x under the observed dependency model.

## What Serialized The Schedule

The schedule was dominated by a small set of hot keys. The top storage key alone contributed 16.542% of observed conflict-key contributions, the top 5 contributed 45.996%, and the top 10 contributed 57.577%.

The hottest contract by conflict contribution was `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913`, touched by 177 transactions. The hottest storage slot by conflict contribution was `0x4200000000000000000000000000000000000006:0x0d52ad225b9f8da090dc37c741705dabc30f648dce00d7b0cab66994a1261ea6`, touched by 60 transactions.

## Where Parallelism Was Available

The observed dependency graph produced 60 deterministic waves with a max wave width of 106 transactions. Canonical worker simulation scaled from 71,655,982 duration units on 1 worker to 16,953,037 on 16 workers, a 4.227x simulated speedup that is essentially at the gas-weighted critical-path bound.

The scheduler ablation found that critical-path priority improved the canonical schedule by 7.460% at 4 workers and reached the critical-path bound at 8 and 16 workers. Gas longest-processing-time priority hurt this block because it delayed shorter transactions that unlocked long dependent chains.

## What Hot Contracts And Slots Dominated

This block's contention was concentrated enough that hot-key-aware execution policy would matter. The top contract and top storage slot listed above explain a meaningful share of conflicts, but the broader pattern is a queueing problem: a few high-fanout keys create long dependency chains while many non-conflicting transactions remain ready.

## What I Would Optimize Next

- Better access-set prediction for recurring hot contracts and slots.
- Hot-slot-aware scheduling that prioritizes transactions which unlock dependent chains.
- Speculative execution with targeted re-execution around known hot keys, rather than broad revalidation.
- Read/write set caching for contracts with stable observed access shapes.
- State-prefetch hints for hot slots and dependent wave frontiers.
- Database read coalescing for repeated hot-slot reads inside the same execution batch.
- Worker balancing by gas-weighted duration, but only after accounting for dependency-unlock priority.
- Conflict-aware batching that isolates high-fanout keys while keeping independent work ready.

## What This Analysis Cannot Prove

This does not prove production TPS, Ggas/s, full EVM replay equivalence, or complete Ethereum access lists. It is an execution-contention study over observed trace-pack accesses and deterministic theoretical scheduling.
