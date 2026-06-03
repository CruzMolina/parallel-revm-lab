# revm Integration Notes

No `revm` adapter is included in this revision.

The core scheduler is implemented against an internal deterministic model because the P0/P1 goal is to prove conflict-aware scheduling, deterministic commit order, re-execution, state hashing, and reproducible reports without leaving broken feature-gated code.

## Current Status

No compile-driven `revm` integration was attempted in the release-repair pass. The repository keeps the internal scheduler clean and avoids committing a token `revm` dependency or feature-gated code without a meaningful smoke path.

Adding an adapter would require mapping EVM execution observations into this lab's `AccessKey` and `TxDelta` model without weakening the all-features build.

## Next Concrete Step

Create a feature-gated adapter crate only after selecting a specific `revm` version and proving a tiny smoke path that compiles with:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

The adapter should remain out of the main scheduler path until it can expose deterministic read/write-like observations cleanly.
