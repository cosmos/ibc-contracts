# Contracts

This directory implements the IBC v2 protocol stack in Solidity. Most entrypoints are upgradeable and deploy per-client/per-user components through beacons so packet handling, tokenization, and message execution can be upgraded independently.

## Core applications
- `ICS26Router.sol` – Core IBC router that registers apps by port id, stores commitments via `IBCStoreUpgradeable`, tracks light clients through `ICS02ClientUpgradeable`, and drives the packet lifecycle (send/recv/ack/timeout) with relayer and admin role gates.
- `ICS20Transfer.sol` – ICS20 fungible token app. It mints/burns `IBCERC20` wrappers for non-native denoms, escrows native tokens per client via `Escrow` beacons, enforces pausing/rate limits, and routes packets through the ICS26 router (supports Permit2 flow).
- `ICS27GMP.sol` – ICS27 general message passing app. It spawns deterministic `ICS27Account` proxies via a beacon, encodes call payloads, and sends GMP packets through ICS26; acknowledgements feed results back to callers.

## Utilities (`utils/`)
- `Escrow.sol` – Per-client token escrow controlled by `ICS20Transfer`; enforces daily rate limits before releasing funds back out.
- `IBCERC20.sol` – Upgradeable ERC20 minted/burned by `ICS20Transfer` to represent inbound IBC denoms; allows one-time custom metadata.
- `ICS27Account.sol` – Minimal execution account used by ICS27 to perform arbitrary calls/value transfers; only the owning ICS27 contract can drive it.
- `ICS02ClientUpgradeable.sol` – Registry/router for light clients; assigns client ids, stores counterparty info, and gates client upgrades via AccessManager roles.
- `IBCStoreUpgradeable.sol` – Commitment store for packet commitments/acks/receipts plus next send sequence tracking; uses `ICS24Host` paths.
- `ICS24Host.sol` and `IBCIdentifiers.sol` – Pure helpers for canonical host paths/keys and identifier validation.
- `ICS20Lib.sol` / `ICS27Lib.sol` – Encoding/version helpers for ICS20 and ICS27 plus utility logic (denom parsing, GMP ack encoding, beacon proxy bytecode for ICS27 accounts).
- `IBCRolesLib.sol` – Shared role ids and selector lists for access-managed permissions across ICS20/ICS26/ICS02.
- `IBCSenderCallbacksLib.sol`, `IBCCallbackReceiver.sol` – Helpers for standardized callback interfaces used by apps.
- `RateLimitUpgradeable.sol` – Reusable per-token, per-day rate limiting mixin used by escrow.
- `RelayerHelper.sol` – Read-only helper for relayers to query packet commitments, receipts, and successful acknowledgements from `ICS26Router`.

## Light clients (`light-clients/`)
- `attestation/AttestationLightClient.sol` – m-of-n attestor-set light client using ECDSA signatures with role-gated proof submission.
- `besu/BesuIBFT2LightClient.sol` – Besu IBFT 2.0 light client for header-validator mode. Uses weak-subjectivity / trusting-period updates, verifies Besu commit seals under the existing YUI + prover sealing-header model plus EVM account/storage proofs, and proves Eureka commitments against the tracked counterparty `ICS26Router` storage.
- `besu/BesuQBFTLightClient.sol` – Besu QBFT light client for header-validator mode with the same proof model and storage verification surface as the IBFT2 wrapper, but QBFT-specific seal-digest rules within that same YUI + prover sealing-header model.
- `ics02-wrapper/ICS02PrecompileWrapper.sol` – Thin adapter that exposes the `ILightClient` interface over the Cosmos EVM ICS02 precompile at a fixed address.
- `sp1-ics07/SP1ICS07Tendermint.sol` – Tendermint light client verified via SP1 programs and verifier contract; supports update, (non)membership proofs, and misbehaviour handling.

### Besu light-client scope
- Supported: Besu **IBFT 2.0** and **QBFT** in **header-validator mode**.
- Verification model: weak subjectivity via **trusting period** and validator-set overlap checks.
- Commit-seal verification: follows the existing **YUI Solidity client + besu-ibc-relay-prover** sealing-header reconstruction model.
- Trusted overlap threshold: requires **strictly greater than one-third** overlap with the trusted validator set, implemented as `floor(n / 3) + 1`, which is intentionally stricter than the current upstream YUI check.
- Proof surface: Besu block headers, commit seals, Ethereum account proofs, and Ethereum storage proofs.
- Counterparty storage model: Eureka `ICS26Router` / `IBCStoreUpgradeable` commitments mapping.
- Not supported in v1: QBFT validator-contract mode, mode transitions, and misbehaviour handling.
- Current fixture status: `test/besu-bft/fixtures/` are synthetic regression fixtures; real Besu-derived golden fixtures remain a follow-up interoperability-confidence improvement.

## Interchain Fungible Tokens (IFT)
- Code reference: find the  `contracts` IFT contract code [here](https://github.com/cosmos/solidity-ibc-eureka/tree/mariuszzak/ift/contracts).
- IFT is an issuer-controlled ERC20 that bridges via ICS27-GMP instead of ICS20. The abstract `IFTBase` burns on send, constructs a mint call for the counterparty IFT, and tracks pending transfers keyed by (clientId, sequence).
- Bridges: `registerIFTBridge` configures a counterparty IFT contract per IBC client along with an `IIFTSendCallConstructor` helper to encode the mint call for that chain.
- Sending: `iftTransfer` burns locally, builds ICS27 `SendCall` to the remote IFT, records `PendingTransfer`, and emits initiation events; default timeout is 15 minutes if not provided.
- Receiving: `iftMint` is callable only by the ICS27-controlled account; it mints locally after verifying the counterparty sender matches the registered bridge and clears pending transfers on ack/timeout callbacks to refund/mint as appropriate.
- Extensibility: implement concrete ERC20 constructors and different `IIFTSendCallConstructor` variants for EVM vs Cosmos SDK token factory flows; access is governed by `AccessManaged` authority roles.

## Interfaces, errors, and message shapes
- `interfaces/` – External interfaces for apps, light clients, stores, rate limiting, pausing, and callbacks.
- `errors/` – Custom error definitions used across the stack for gas-efficient reverts.
- `msgs/` – ABI-encoded packet/message structs for ICS02/20/26/27, light clients, and app callbacks.

## Messages and docs
- `PICKUP.md` – Notes for maintainers on pending tasks.
- `light-clients/attestation/IBC_ATTESTOR_DESIGN.md` & `ics02-wrapper/README.md` – Additional design notes for their respective light clients.
