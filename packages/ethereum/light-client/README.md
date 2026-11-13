<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# `ethereum-light-client`

> [!CAUTION]
> ⚠ The Ethereum Light Client is currently under heavy development, and is not ready for use.

This is the stateless verification implementation of the ethereum light client. It contains all the core logic for verifying ethereum consensus, proving state (verify (non)memebership) and the headers submitted to update the light client.
The state is handled by the `CosmWasm` implementation in `ibc-solidity/programs/cw-ics08-wasm-eth`.

## Acknowledgements

This work is based on the ethereum light client created by [Union](http://github.com/unionlabs/union/)
