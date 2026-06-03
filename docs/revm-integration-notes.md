# revm Integration Notes

`crates/revm-smoke` is included and compile-tested. It is deliberately small: it proves this workspace can execute real EVM bytecode through `revm`, record storage opcodes with an inspector, emit a compact trace pack, and feed those observations into the shared dossier analyzer.

## Current Status

- Crate: `crates/revm-smoke`
- Dependency: `revm = "=40.0.3"`, `default-features = false`, `features = ["std"]`
- MSRV impact: `revm 40.0.3` declares `rust-version = 1.91.0`, so the workspace `rust-version` is `1.91`
- Database: in-memory `CacheDB<EmptyDB>`
- Execution API: `Context::mainnet().with_db(...).build_mainnet_with_inspector(...).inspect_tx(tx)`

## What The Smoke Path Does

- Executes a counter bytecode fixture that reads and writes one storage slot.
- Executes independent storage write fixtures that touch different slots.
- Executes a hot-slot fixture that creates contention.
- Records `SLOAD` and `SSTORE` in an inspector from opcode, target address, and stack.
- Converts inspector observations into `BlockAccessTrace`.
- Converts the same observations into a trace pack via `smoke_trace_pack`.
- Runs the analyzer and asserts expected conflict/wave behavior.

## What It Does Not Do

- It is not a full EVM block replay adapter.
- It is not a full opcode tracer.
- It does not claim account, balance, nonce, or code read coverage.
- It does not use live RPC.

## Validation

```sh
cargo test -p parallel-revm-lab-revm-smoke --all-features
cargo run -p parallel-revm-lab-revm-smoke --example emit_trace_pack
```

The full workspace validation also runs this crate through `cargo test --workspace --all-features`.

## Next Concrete Step

Extend the inspector to record account, balance, nonce, code, call/create, and selfdestruct observations. Until then, revm smoke reports remain storage-access lower bounds.
