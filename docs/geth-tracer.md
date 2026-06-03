# Geth Storage Access Tracer

`tracers/geth-storage-access-tracer.js` is a compact custom Geth JavaScript tracer for storage contention analysis.

It is designed for:

- `debug_traceTransaction`
- providers or local nodes that support custom JavaScript tracers
- compact trace-pack collection, not full opcode dumps

Official Geth references:

- [JavaScript tracing tutorial](https://geth.ethereum.org/docs/developers/evm-tracing/javascript-tutorial)
- [Custom EVM tracer reference](https://geth.ethereum.org/docs/developers/evm-tracing/custom-tracer)
- [Basic traces and trace-size caveats](https://geth.ethereum.org/docs/developers/evm-tracing/basic-traces)

## What It Records

The tracer watches each EVM step and records:

- `SLOAD` as a storage `read`
- `SSTORE` as a storage `write`
- current contract address via `log.contract.getAddress()`
- storage slot via `log.stack.peek(0)`
- program counter via `log.getPC()`
- depth via `log.getDepth()`
- gas remaining via `log.getGas()`
- faults reported during tracing

It returns:

```json
{
  "accesses": [],
  "faults": [],
  "warnings": []
}
```

## Manual Use

With a local Geth node exposing the debug namespace:

```sh
curl -H "Content-Type: application/json" \
  --data '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"debug_traceTransaction",
    "params":[
      "0xTRANSACTION_HASH",
      {
        "tracer": "<contents of tracers/geth-storage-access-tracer.js>",
        "timeout": "20s"
      }
    ]
  }' \
  http://localhost:8545
```

The CLI wraps this call in:

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

## Provider Caveats

Not every provider supports:

- `debug_traceTransaction`
- custom JavaScript tracers
- historical state for the requested block
- enough timeout or payload size for large transactions

Run `--dry-run` first to check every requested block header and transaction count. A successful dry run does not prove tracing support.

## Security Notes

- Never paste RPC URLs, API keys, bearer tokens, or private keys into tracer files.
- Pass secrets through environment variables or `--rpc-url`.
- The CLI redacts full HTTP(S) URLs from collector errors before printing them.

## Trace Size

Full `structLogs` can become very large. This tracer returns compact storage access observations so the committed trace-pack format can stay reviewable. That compactness is also a limitation: account, balance, nonce, code, call, create, and selfdestruct observations are not complete.
