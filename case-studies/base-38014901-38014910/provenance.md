# Base 38014901-38014910 Provenance

No full-range Base trace pack is committed for this case study.

- Status: real 25-transaction block sample committed separately; full range not traced.
- Local capability command: `cargo run -p parallel-revm-lab -- rpc-capability-check --chain base --block 38014901`.
- Local result after fixing the probe tracer: block fetch, receipts, custom JavaScript tracing, and struct-log tracing succeeded.
- Required provider capability: Base RPC with `debug_traceTransaction`, preferably custom Geth JavaScript tracer support.
- Fallback capability: Geth-style struct logs, if compact custom tracer support is unavailable.
- Full-range dry run: 3,676 transactions across blocks `38014901-38014910`.

Do not cite this directory as a full-range analysis until a compact trace pack has been collected from a debug-capable Base RPC endpoint and the generated dossier has been reviewed for coverage, warnings, and provider limitations. Cite `case-studies/base-38014901-real-sample/` for the committed 25-transaction real sample.
