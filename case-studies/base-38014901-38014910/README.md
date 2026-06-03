# Base 38014901-38014910 Reproduction Notes

This directory intentionally contains instructions only. No committed report in this directory is claimed to be real Base data.

The target public range is:

- start block: `38014901`
- end block: `38014910`

To collect real data, provide a Base RPC endpoint that supports `debug_traceTransaction` with custom Geth JavaScript tracers. This environment did not provide `BASE_RPC_URL`, so no real Base trace pack or real Base dossier was produced in this pass.

## Capability Check

```sh
cargo run -p parallel-revm-lab -- rpc-capability-check \
  --chain base \
  --block 38014901 \
  --rpc-url "$BASE_RPC_URL"
```

Local result on June 3, 2026:

```text
missing RPC URL for rpc-capability-check: pass --rpc-url or set BASE_RPC_URL / ETH_RPC_URL
```

The command checks block availability, transaction presence, receipt access, `debug_traceTransaction`, custom JavaScript tracer support, and struct-log fallback support. It redacts RPC URLs and tokens in provider errors.

## Collect

Collect a small sample first:

```sh
cargo run -p parallel-revm-lab -- collect-block-range \
  --chain base \
  --start-block 38014901 \
  --end-block 38014901 \
  --rpc-url "$BASE_RPC_URL" \
  --tracer geth-js-storage \
  --out trace-packs/base-38014901-real-sample \
  --max-transactions 25 \
  --resume
```

If the sample succeeds and remains compact, collect the full range:

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

Use `--dry-run` first to check every requested block header and transaction count without tracing transactions:

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

Sample analysis:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-real-sample \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-real-sample/dossier.json \
  --markdown case-studies/base-38014901-real-sample/executive-summary.md \
  --html case-studies/base-38014901-real-sample/dossier.html \
  --trace case-studies/base-38014901-real-sample/schedule.trace.json
```

Full-range analysis:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-38014910 \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-38014910/dossier.json \
  --markdown case-studies/base-38014901-38014910/summary.md \
  --html case-studies/base-38014901-38014910/dossier.html \
  --trace case-studies/base-38014901-38014910/schedule.trace.json
```

Observed access hints:

```sh
cargo run -p parallel-revm-lab -- recommend-access-lists \
  --trace-dir trace-packs/base-38014901-real-sample \
  --out case-studies/base-38014901-real-sample/access-hints.json \
  --markdown case-studies/base-38014901-real-sample/access-hints.md
```

## Provenance Rules

- A report generated from `trace-packs/base-38014901-real-sample` or `trace-packs/base-38014901-38014910` should be labeled `user-collected RPC trace pack`.
- A partial sample must report included transaction coverage from `source_tx_count`.
- Do not commit raw provider dumps or huge opcode traces.
- Do not commit RPC URLs, API keys, or environment files.
- Do not describe results as real Base data unless the trace pack was actually collected from a debug-capable Base RPC endpoint.

## Current Status

Real Base collection is blocked in this environment by missing `BASE_RPC_URL`. The committed proof path is `case-studies/demo-trace-pack/`.
