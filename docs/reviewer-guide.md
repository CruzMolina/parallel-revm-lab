# Reviewer Guide

This guide answers the skeptical questions a Rust protocol or execution-client reviewer is likely to ask first.

## Does This Replay EVM Blocks?

No. The real Base dossier analyzes observed SLOAD/SSTORE-style storage topology from a compact trace pack. It does not execute/replay full EVM state transitions for Base block `38014901`.

The `revm-smoke` crate does execute small real EVM bytecode fixtures through `revm`, but that path is deliberately a bridge smoke test, not a full EVM block/state replay adapter.

## Are These Production Speedups?

No. Scheduler ablations are theoretical schedules over observed dependencies with receipt gas used as deterministic duration. The trace-derived benchmark maps real access topology into this repository's synthetic execution model. Those artifacts are useful for reasoning about contention, but they are not TPS, Ggas/s, or production-client benchmark claims.

## Is The Trace Complete?

The committed full-block trace pack captures storage observations available through the custom Geth JavaScript tracer and the provider used for collection. It is not a complete account, balance, nonce, code, call/create, or selfdestruct trace. The reports preserve per-transaction warnings in JSON and show grouped warning summaries in Markdown/HTML.

## Why Does Overlap Exceed Write Conflicts?

`overlapping_tx_percentage` counts transactions that share at least one observed access key with another transaction. Shared keys can be read-compatible. Serializing dependencies are narrower: they arise from write/write or write/read conflicts in canonical transaction order. That is why Base block `38014901` can show 92.431% overlapping transactions while write-related conflict pairs are 5.327%.

## What Does revm Prove?

The `revm-smoke` path proves this workspace can execute real EVM bytecode through `revm`, capture SLOAD/SSTORE observations with an inspector, emit a trace pack, and feed that trace pack into the same analyzer. It does not prove full EVM block/state replay or complete dependency extraction.

## Why Add Contract Labels?

Labels are convenience metadata for readability. The current label file contains only verified static hints for Base USDC and Base WETH. Unknown addresses render as `unknown`. Labels do not affect conflict metrics, waves, critical paths, scheduling, or sequential-equivalence tests.

## What Would Be Next In Production?

- Broader access observation: account, balance, nonce, code, call/create, and selfdestruct coverage.
- Access-set prediction for stable contract call shapes.
- Hot-key-aware scheduling with critical-path or dependency-unlock priority.
- State prefetch hints and database read coalescing around repeated hot keys.
- Targeted speculative re-execution instead of broad validation fallback.
- Integration with a real execution client and measured end-to-end state transition costs.
