# Failure Log

See `docs/engineering-log.md` for the current validation history and upgrade decisions. This file keeps the original focused failure notes for quick review.

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

## Correctness Docs Outran Property Tests

- Symptom: `docs/correctness.md` said property tests compared full final `State` values, but the proptest assertions only compared final state hashes.
- Root cause: fixed examples compared full state, while the randomized property test stopped at the hash invariant.
- Fix: property tests now compare both state hashes and full final `State` values for access-list and optimistic execution.
- Test added: strengthened existing `random_small_workloads_match_sequential` proptest assertions.

## Ambiguous Conflict Metric

- Symptom: reports used a single `conflicts_detected` field for both access-list scheduler behavior and optimistic declared conflict counts.
- Root cause: scheduler deferrals, declared access-set conflicts, and optimistic validation failures were collapsed into one field.
- Fix: report schema now splits `declared_conflict_pairs`, `scheduler_deferrals`, `validation_failures`, `reexecuted_txs`, `wave_count`, and `max_wave_width`.
- Test added: benchmark report test asserts split metric consistency.

## Nonce Write Cast Could Wrap

- Symptom: generic nonce writes cast signed values to `u64`, allowing negative values to wrap into huge nonces.
- Root cause: unchecked `as u64` conversion in `State::write`.
- Fix: negative nonce writes clamp to zero and oversized writes saturate to `u64::MAX`.
- Test added: `nonce_write_saturates_instead_of_wrapping`.

## Incomplete Reads Were Overstated

- Symptom: analyzer warnings initially described incomplete-read reports as conservative.
- Root cause: missing reads can hide conflicts, so the analysis is a lower bound instead.
- Fix: reports now use `declared-read-write-lower-bound` and explicit lower-bound warnings.
- Test added: committed fixture analyzer test checks the warning.

## revm MSRV Mismatch

- Symptom: adding `revm 40.0.3` would make the old workspace `rust-version = "1.82"` claim false.
- Root cause: `revm 40.0.3` declares `rust-version = 1.91.0`.
- Fix: workspace `rust-version` is now `1.91`.
- Test added: `cargo test -p parallel-revm-lab-revm-smoke --all-features`.

## Mnemonic Fixture Addresses Were Not Hex

- Symptom: the public fixture used mnemonic strings such as `0xpool...`, which are not valid EVM addresses.
- Root cause: the first fixture optimized readability over literal EVM hex validity.
- Fix: replaced addresses with valid 20-byte hex values and added validation for addresses, storage keys, and tx hashes.
- Tests added: invalid address, invalid storage key, invalid tx hash, and committed fixture validation.

## revm Smoke Observations Were Fixture-Derived

- Symptom: the smoke crate executed revm bytecode but derived storage observations from the fixture enum.
- Root cause: the first revm bridge prioritized a compiling smoke path before implementing inspector hooks.
- Fix: added a revm inspector that records `SLOAD` and `SSTORE` from opcode, target address, and stack.
- Tests added: existing revm conflict/wave tests now run through inspector-captured observations.
