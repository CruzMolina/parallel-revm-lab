# Provenance

## Dataset

- Chain: Base mainnet.
- Block range: `38014901-38014901`.
- Coverage: 436 of 436 transactions.
- Trace pack: `trace-packs/base-38014901-full`.
- Tracer: `geth-js-storage`.
- Normalized trace size: about 9.1 MB.
- Generated case-study size: about 2.0 MB.

## Collection

The endpoint was supplied through `BASE_RPC_URL`; the URL is not stored in repository files.

Capability check:

```sh
cargo run -p parallel-revm-lab -- rpc-capability-check \
  --chain base \
  --block 38014901
```

Collection:

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014901 \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-full \
  --resume
```

Analysis:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-execution-dossier/dossier.json \
  --markdown case-studies/base-38014901-execution-dossier/executive-summary.md \
  --html case-studies/base-38014901-execution-dossier/dossier.html \
  --trace case-studies/base-38014901-execution-dossier/schedule.trace.json
```

Observed access hints:

```sh
cargo run -p parallel-revm-lab -- recommend-access-lists \
  --trace-dir trace-packs/base-38014901-full \
  --out case-studies/base-38014901-execution-dossier/access-hints.json \
  --markdown case-studies/base-38014901-execution-dossier/access-hints.md
```

Trace-derived synthetic benchmark:

```sh
cargo run --release -p parallel-revm-lab -- bench-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --mode all \
  --threads 1,2,4,8,16 \
  --vm-steps-per-gas 1 \
  --out case-studies/base-38014901-execution-dossier/trace-derived-bench.json
```

## Limitations

- The compact tracer records observed storage access around SLOAD/SSTORE-style events. It does not claim complete account, balance, nonce, code, precompile, or full dynamic access coverage.
- The scheduling model is theoretical: it uses observed read/write sets and gas-used duration weights; it does not replay the EVM.
- The trace-derived benchmark maps observed access topology into the repository's deterministic toy execution model. It is useful for scheduler experiments, not production throughput claims.
- Provider support can vary across endpoints even when the same JSON-RPC method names are present.
