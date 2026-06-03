# revm Integration Notes

`crates/revm-smoke` is included and compile-tested. It is deliberately small: it proves this workspace can execute real EVM bytecode through `revm` and feed deterministic read/write-like observations into the shared trace analyzer.

## Current Status

- Crate: `crates/revm-smoke`
- Dependency: `revm = "=40.0.3"`, `default-features = false`, `features = ["std"]`
- MSRV impact: `revm 40.0.3` declares `rust-version = 1.91.0`, so the workspace `rust-version` is `1.91`
- Database: in-memory `CacheDB<EmptyDB>`
- Execution API: `Context::mainnet().with_db(...).build_mainnet().transact(tx)`

## What The Smoke Path Does

- Executes a counter bytecode fixture that reads and writes one storage slot.
- Executes independent storage write fixtures that touch different slots.
- Executes a hot-slot fixture that creates contention.
- Converts the known fixture behavior into `BlockAccessTrace`.
- Runs the analyzer and asserts expected conflict/wave behavior.

## What It Does Not Do

- It is not a full EVM block replay adapter.
- It is not a general revm inspector or opcode tracer.
- It does not claim complete read coverage for arbitrary bytecode.
- It does not use live RPC.

## Validation

```sh
cargo test -p parallel-revm-lab-revm-smoke --all-features
```

The full workspace validation also runs this crate through `cargo test --workspace --all-features`.

## Next Concrete Step

Add a revm inspector that records `SLOAD`, `SSTORE`, account, balance, nonce, and code observations from arbitrary bytecode. Keep fixture-derived observations until inspector coverage is complete enough to avoid under-reporting conflicts.
