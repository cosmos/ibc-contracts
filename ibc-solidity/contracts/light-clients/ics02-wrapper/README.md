# ICS-02 Precompile Wrapper

This solidity IBC light client contract serves as a wrapper for the [ICS02 Precompile](https://github.com/cosmos/evm/tree/main/precompiles/ics02) in [`cosmos/evm`](https://github.com/cosmos/evm/).

## Overview

This contract, when deployed on a Cosmos SDK-based blockchain with EVM support, allows users to interact with a IBC-Go light client through a familiar Solidity interface where each light client instance corresponds to a single contract. This contract is used to integrate `ibc-contracts` with Cosmos SDK-based chains that support the ICS02 precompile.


> [!WARNING]
> This contract is specifically designed to work with the ICS-02 precompile available in Cosmos SDK-based chains.
> Only deploy this contract on Cosmos SDK-based chains that have the ICS-02 precompile enabled.
