# Base 38014901-38014910 Reproduction Notes

This directory intentionally contains instructions only. No committed report in this directory is claimed to be real Base data.

The target public range is:

- start block: `38014901`
- end block: `38014910`

To collect real data, provide a Base RPC endpoint that supports `debug_traceTransaction` with custom Geth JavaScript tracers.

## Collect

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014910 \
  --rpc-url "$BASE_RPC_URL" \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-38014910 \
  --resume
```

Use `--dry-run` first to check block availability without tracing transactions:

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014910 \
  --rpc-url "$BASE_RPC_URL" \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-38014910 \
  --dry-run
```

## Analyze

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-38014910 \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-38014910/dossier.json \
  --markdown case-studies/base-38014901-38014910/summary.md \
  --html case-studies/base-38014901-38014910/dossier.html \
  --trace case-studies/base-38014901-38014910/schedule.trace.json
```

## Provenance Rules

- A report generated from `trace-packs/base-38014901-38014910` should be labeled `user-collected RPC trace pack`.
- Do not commit raw provider dumps or huge opcode traces.
- Do not commit RPC URLs, API keys, or environment files.
- Do not describe results as real Base data unless the trace pack was actually collected from a debug-capable Base RPC endpoint.

## Current Status

Real Base collection is pending a user-supplied debug-capable RPC endpoint. The committed proof path is `case-studies/demo-trace-pack/`.
