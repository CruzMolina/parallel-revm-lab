# Base 38014901-38014910 Provenance

No real Base trace pack is committed for this case study.

- Status: blocked in this environment by missing `BASE_RPC_URL`.
- Local capability command: `cargo run -p parallel-revm-lab -- rpc-capability-check --chain base --block 38014901`.
- Local result: failed before network access because no `BASE_RPC_URL` or `ETH_RPC_URL` was set.
- Required provider capability: Base RPC with `debug_traceTransaction`, preferably custom Geth JavaScript tracer support.
- Fallback capability: Geth-style struct logs, if compact custom tracer support is unavailable.

Do not cite this directory as real-chain data until a compact trace pack has been collected from a debug-capable Base RPC endpoint and the generated dossier has been reviewed for coverage, warnings, and provider limitations.
