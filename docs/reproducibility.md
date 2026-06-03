# Reproducibility

This repository has two review paths: an offline path that needs no secrets and a live collection path that needs a debug-capable Base RPC endpoint.

## Offline Review Path

These commands work from committed artifacts without RPC credentials.

Analyze the committed full Base block trace pack:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-execution-dossier/dossier.json \
  --markdown case-studies/base-38014901-execution-dossier/executive-summary.md \
  --html case-studies/base-38014901-execution-dossier/dossier.html \
  --trace case-studies/base-38014901-execution-dossier/schedule.trace.json
```

Run synthetic scheduler verification:

```sh
cargo run -p parallel-revm-lab -- verify \
  --workload mixed \
  --txs 100 \
  --conflicts 0.0,0.2,0.5,0.7,0.95 \
  --threads 1,2,4 \
  --seed-start 1 \
  --seed-end 20
```

Run the trace-derived synthetic benchmark:

```sh
cargo run -p parallel-revm-lab -- bench-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --mode all \
  --threads 1,2,4,8 \
  --vm-steps-per-gas 1 \
  --out case-studies/base-38014901-execution-dossier/trace-derived-bench.json
```

Run the `revm` smoke bridge:

```sh
cargo test -p parallel-revm-lab-revm-smoke --all-features
```

For a fast scripted path:

```sh
just reviewer-demo
```

For a broader no-RPC validation path:

```sh
just reviewer-validate
```

## Real Collection Path

Live collection requires a Base RPC endpoint that supports `debug_traceTransaction`, preferably with custom JavaScript tracers.

Capability check:

```sh
cargo run -p parallel-revm-lab -- rpc-capability-check \
  --chain base \
  --block 38014901 \
  --rpc-url "$BASE_RPC_URL"
```

Dry-run the public range without tracing transactions:

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014910 \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-38014910 \
  --dry-run
```

Collect the committed one-block artifact:

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014901 \
  --rpc-url "$BASE_RPC_URL" \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-full \
  --resume
```

`--resume` validates already-written block files under `blocks/` and continues from the remaining blocks. The collector writes compact normalized block JSON, not raw opcode dumps.

## Expected Artifacts

- Trace pack: `trace-packs/base-38014901-full/`
- Dossier JSON: `case-studies/base-38014901-execution-dossier/dossier.json`
- Dossier Markdown: `case-studies/base-38014901-execution-dossier/executive-summary.md`
- Dossier HTML: `case-studies/base-38014901-execution-dossier/dossier.html`
- Access hints: `case-studies/base-38014901-execution-dossier/access-hints.json`
- Schedule trace: `case-studies/base-38014901-execution-dossier/schedule.trace.json`

## What Can Differ

- Rust version and optimizer behavior.
- CPU timing in synthetic benchmarks.
- RPC provider behavior for live collection.
- Optional convenience labels in `labels/base-known-contracts.json`.

## What Should Not Differ

- Deterministic conflict graph from the committed trace pack.
- Wave count, max wave width, gas critical path, and worker simulation metrics from the committed trace pack.
- Sequential equivalence tests for access-list and optimistic execution modes.
- Stable generated metrics when inputs and label file are unchanged.

## Security

- Never commit RPC URLs, bearer tokens, API keys, or environment files.
- Collector and capability errors redact HTTP(S) RPC URLs and common bearer/API token forms.
- Do not commit raw full opcode traces; committed trace packs should stay compact and normalized.
