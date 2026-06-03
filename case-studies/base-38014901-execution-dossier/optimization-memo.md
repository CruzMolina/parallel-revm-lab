# Optimization Memo

## Dataset And Provenance

This memo interprets the committed full-block trace pack for Base block `38014901`. The dataset covers 436 of 436 block transactions, 71,655,982 gas, 27,034 observed access events, 386 unique contracts, and 4,321 unique storage slots. It was collected with the repository's compact Geth JavaScript storage tracer and analyzed from the normalized trace pack at `trace-packs/base-38014901-full`.

The labels used here are convenience metadata only. The two labels currently applied are verified static hints: Base USDC at `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913` and Base WETH at `0x4200000000000000000000000000000000000006`. Unknown addresses remain labeled `unknown`; labels do not affect dependency construction, conflict counts, waves, or critical-path metrics.

## Headline Finding

This block has high shared-key overlap but much less write-related serialization than the overlap number alone might imply. The analyzer found 403 of 436 transactions, or 92.431%, shared at least one observed key with another transaction. Pairwise write-related conflicts were 5,052 of 94,830 possible transaction pairs, or 5.327%.

That gap is the central lesson of this dossier. A high-overlap workload is not automatically a fully serialized workload. The useful engineering question is which shared keys create write/read or write/write dependency edges, how those edges shape waves, and how much gas sits on the critical path.

## Overlap Vs Serialization

The model separates read-compatible overlap from serializing dependencies. Read/read overlap can be common in EVM workloads and still admit parallel execution. Dependencies arise when a write conflicts with a later read or write, preserving canonical transaction order. Under that model, this block produced 60 deterministic waves with a max wave width of 106.

That wave shape says two things at once. First, the block has real hot-state structure; it is not embarrassingly parallel. Second, there is still substantial parallel work outside the critical path. A scheduler that treats all overlap as serialization would leave work on the table. A scheduler that ignores the hot keys would overestimate available parallelism.

## Critical Path And Scheduler Ablation

Using receipt gas as deterministic duration, the gas-weighted critical path was 16,951,990 out of 71,655,982 total gas covered. That implies a theoretical gas ceiling of 4.227x for the observed dependency graph. Canonical worker simulation reached 3.701x at 4 workers and 4.227x at 16 workers, essentially landing on the critical-path bound.

The scheduler ablation is the most actionable result. Critical-path ready-queue priority improved the 4-worker schedule from canonical's 19,359,165 duration units to 17,914,903, a 7.460% improvement at the same worker count. At 8 and 16 workers, critical-path priority reached the lower bound of 16,951,990.

Gas longest-processing-time priority did not help this block. At 4 workers it was 17.229% worse than canonical. The reason is mechanical: the longest ready transaction is not always the transaction that unlocks the most future work. If a short transaction gates a long dependent chain, delaying it can increase makespan even while workers are busy. For execution-client scheduling, duration estimates need to be paired with dependency-unlock value.

## Hot-State Concentration

The hottest contract by conflict contribution was Base USDC, touched by 177 transactions, with 208 unique slots and 2,543 conflict contributions. The hottest storage slot by conflict contribution was under the Base WETH address, touched by 60 transactions, with 1,704 conflict contributions.

Contention was concentrated: the top 1 key contributed 16.542% of observed conflict-key contributions, the top 5 contributed 45.996%, and the top 10 contributed 57.577%. This is exactly the kind of shape where hot-key-aware execution policy can matter. The goal is not just to find conflicts; it is to avoid letting a few high-fanout keys quietly dominate the whole worker pool.

## Optimization Recommendations

The first optimization target is access-set prediction for recurring hot contracts and slots. If a client can predict likely read/write sets for stable call shapes, it can schedule around hot keys earlier and reserve speculation for the uncertain edges.

The second target is hot-slot-aware scheduling. A ready transaction should be prioritized not only by gas estimate, but by whether it unlocks a long dependent chain. The critical-path ablation is a small theoretical version of that idea.

The third target is targeted speculative execution. Rather than broad re-execution, speculative validation should concentrate around the hot keys that actually cause invalidations. That pairs naturally with read/write set caching and conflict-aware batching.

The fourth target is state access efficiency: state-prefetch hints, database read coalescing for repeated hot-slot reads, and batch-level caching for contracts with stable access shapes. If many transactions touch the same small set of keys, storage scheduling and database behavior are part of the same performance problem.

## What This Does Not Prove

This does not prove production TPS, Ggas/s, full EVM state-transition replay, or complete Ethereum access lists. The tracer observes storage access behavior available through the custom tracer and provider. The scheduler results are theoretical gas-duration schedules over observed dependencies. The trace-derived benchmark is synthetic execution over real access topology, not a production execution-client benchmark.
