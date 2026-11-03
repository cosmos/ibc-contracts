<!-- SPDX-License-Identifier: Apache-2.0 -->

# Architecture Decision Records

The architecture decision records for `ibc-contracts` have moved to [`cosmos/ibc/spec/adr`](https://github.com/cosmos/ibc/tree/main/spec/adr), which holds the design decisions for the Interchain Standards and for cosmos/ibc, cosmos/ibc-go and cosmos/ibc-contracts in one numbered series. Their history before the move is in this repository.

| ADR | Title | Previously |
| --- | --- | --- |
| [028](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-028-solana-storage-architecture.md) | Solana IBC Storage Architecture | `solana/solana-storage-architecture.md` |
| [029](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-029-solana-upgradability.md) | Solana IBC Contract Upgradability | `solana/solana-upgradability.md` |
| [030](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-030-solana-ics27-gmp-architecture.md) | ICS27 General Message Passing (GMP) Architecture for Solana | `solana/solana-ics27-gmp-architecture.md` |
| [031](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-031-solana-ift-architecture.md) | Interchain Fungible Token (IFT) Architecture for Solana | `solana/solana-ift-architecture.md` |
| [032](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-032-solana-attestation-light-client.md) | Solana Attestation Light Client | `solana/solana-attestation-light-client.md` |
| [033](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-033-proof-api-version-specific-compatibility-modules.md) | Version-Specific Proof API Compatibility Modules | `proof-api/version-specific-compatibility-modules.md` |
| [034](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-034-solidity-ift-ratelimit.md) | Rate Limiting in IFT in Solidity | `solidity/ift-ratelimit.md` |
| [035](https://github.com/cosmos/ibc/blob/main/spec/adr/adr-035-solidity-pausability.md) | Pausing and Freezing in IBC-Solidity | `solidity/pausability.md` |

To propose an architecture decision for `ibc-contracts`, follow the [ADR process](https://github.com/cosmos/ibc/blob/main/spec/adr/README.md#process) and open the pull request in `cosmos/ibc` with `ibc-contracts/solidity`, `ibc-contracts/solana` or `ibc-contracts/proof-api` in its scope.
