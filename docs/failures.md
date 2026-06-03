# Failure Log

## Missing CLI Dependencies

- Symptom: `cargo test --workspace --all-features` failed with unresolved `serde` and `parallel_revm_lab_model` in `crates/cli/src/main.rs`.
- Root cause: the CLI wrote generic JSON and formatted `AccessKey` values but did not declare direct dependencies on the crates that define those symbols.
- Fix: added `serde` and `parallel-revm-lab-model` to `crates/cli/Cargo.toml`.
- Test added: no new test was needed; existing CLI smoke tests cover the binary after compilation.

## Access-List Reordered Deferred Conflicts

- Symptom: executor tests found access-list final state hashes diverging from sequential hashes on mixed and high-contention workloads.
- Root cause: the wave builder admitted later transactions that were independent of the current wave but conflicted with earlier transactions already deferred to the next wave, effectively moving them before a conflicting predecessor.
- Fix: track access sets for earlier deferred transactions and prevent later conflicting transactions from joining the current wave.
- Test added: existing fixed, high-contention, and proptest sequential-equivalence tests cover the repaired invariant. The generated proptest regression seed is checked in under `crates/executor/proptest-regressions/lib.txt`.
