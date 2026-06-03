# Contract Labels

`base-known-contracts.json` contains static convenience labels used only to make reports easier to read.

Rules:

- Labels are not analysis inputs.
- Unknown addresses render as `unknown`.
- Labels must not change conflicts, waves, critical paths, scheduling, or benchmark results.
- Add only verified labels.

Current sources:

- Base USDC: Circle USDC contract address documentation for Base, `https://developers.circle.com/stablecoins/usdc-contract-addresses`.
- Base WETH: Base contract documentation for the WETH9 predeploy, `https://docs.base.org/base-chain/network-information/base-contracts`.
