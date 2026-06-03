# Base 38014901-38014910 Reproduction Notes

This directory intentionally contains full-range reproduction instructions only. No committed report in this directory is claimed to be the full `38014901-38014910` Base range.

The target public range is:

- start block: `38014901`
- end block: `38014910`

To collect real data, provide a Base RPC endpoint that supports `debug_traceTransaction` with custom Geth JavaScript tracers. A full-block trace-backed analysis of block `38014901` is committed under `case-studies/base-38014901-execution-dossier/`.

## Capability Check

```sh
cargo run -p parallel-revm-lab -- rpc-capability-check \
  --chain base \
  --block 38014901 \
  --rpc-url "$BASE_RPC_URL"
```

Local result on June 3, 2026 after fixing the probe tracer:

```text
rpc capability base block 38014901 status=ok txs=436 receipts=true js_tracer=true struct_logs=true
```

The command checks block availability, transaction presence, receipt access, `debug_traceTransaction`, custom JavaScript tracer support, and struct-log fallback support. It redacts RPC URLs and tokens in provider errors.

## Collect

Collect the committed full-block case-study trace first:

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

If the one-block trace succeeds and remains compact, collect the full range:

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

One-block case-study analysis:

```sh
cargo run -p parallel-revm-lab -- analyze-trace-pack \
  --trace-dir trace-packs/base-38014901-full \
  --workers 1,2,4,8,16 \
  --out case-studies/base-38014901-execution-dossier/dossier.json \
  --markdown case-studies/base-38014901-execution-dossier/executive-summary.md \
  --html case-studies/base-38014901-execution-dossier/dossier.html \
  --trace case-studies/base-38014901-execution-dossier/schedule.trace.json
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
  --trace-dir trace-packs/base-38014901-full \
  --out case-studies/base-38014901-execution-dossier/access-hints.json \
  --markdown case-studies/base-38014901-execution-dossier/access-hints.md
```

## Provenance Rules

- A report generated from `trace-packs/base-38014901-full` or `trace-packs/base-38014901-38014910` should be labeled `user-collected RPC trace pack`.
- A partial sample must report included transaction coverage from `source_tx_count`.
- Do not commit raw provider dumps or huge opcode traces.
- Do not commit RPC URLs, API keys, or environment files.
- Do not describe results as real Base data unless the trace pack was actually collected from a debug-capable Base RPC endpoint.

## Current Status

Real Base full-block collection succeeded for block `38014901`; see `case-studies/base-38014901-execution-dossier/`.

Full-range tracing is not committed in this directory. A dry run found 3,676 transactions across blocks `38014901-38014910`, so tracing the entire range would require thousands of debug trace calls and likely produce much larger artifacts.
