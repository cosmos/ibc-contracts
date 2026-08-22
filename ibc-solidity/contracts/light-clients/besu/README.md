# Besu IBFT 2.0 / QBFT Light Clients

This module contains two Solidity light clients for Besu BFT chains:

- `BesuIBFT2LightClient.sol`
- `BesuQBFTLightClient.sol`

Both wrappers share the same storage model and proof surface through `BesuLightClientBase.sol`. They differ only in how the commit-seal signing digest is reconstructed from the raw Besu header.

## Verification model notes

- Commit-seal verification follows the existing **YUI Solidity client + besu-ibc-relay-prover** model: reconstruct the sealing header by rewriting `extraData` into the protocol-specific signing form, then recover commit-seal signers from the `keccak256(RLP(header))` digest.
- This module does **not** claim to independently rederive a distinct Besu network-level consensus-message payload beyond that established YUI/prover model.
- Trusted overlap is intentionally **strictly greater than one-third** of the trusted validator set, implemented as `floor(n / 3) + 1`. This is intentionally stricter than the current upstream YUI overlap check.

## Supported scope

- Besu **IBFT 2.0**
- Besu **QBFT**
- **Header-validator mode only**
- Weak-subjectivity / **trusting-period** verification
- Ethereum **account proofs** and **storage proofs**
- Eureka commitment verification against the counterparty `ICS26Router` proxy account

## Out of scope in v1

- QBFT validator-contract mode
- Mode transitions
- Misbehaviour evidence handling
- Frozen-client machinery

## Constructor

Both wrappers take the same constructor arguments:

```solidity
constructor(
    address ibcRouter,
    uint64 initialTrustedHeight,
    uint64 initialTrustedTimestamp,
    bytes32 initialTrustedStorageRoot,
    address[] memory initialTrustedValidators,
    uint64 trustingPeriod,
    uint64 maxClockDrift,
    address roleManager
)
```

- `ibcRouter`: counterparty `ICS26Router` proxy address whose account/storage proofs are tracked.
- `initialTrustedHeight`: trusted Besu block number. Revision number is always `0`.
- `initialTrustedTimestamp`: trusted header timestamp in seconds.
- `initialTrustedStorageRoot`: storage root of the tracked `ICS26Router` account at `initialTrustedHeight`.
- `initialTrustedValidators`: validator set trusted at `initialTrustedHeight`.
- `trustingPeriod`: weak-subjectivity window in seconds. `0` means no expiry.
- `maxClockDrift`: allowed future drift for submitted headers in seconds.
- `roleManager`: if non-zero, receives admin and `PROOF_SUBMITTER_ROLE`; if zero, proof submission is open to anyone through the zero-address sentinel.

## `updateClient(bytes)` ABI

`updateClient` expects `abi.encode(IBesuLightClientMsgs.MsgUpdateClient)`:

```solidity
struct MsgUpdateClient {
    bytes headerRlp;
    IICS02ClientMsgs.Height trustedHeight;
    bytes accountProof;
}
```

- `headerRlp`: full raw Besu block header RLP, including `extraData` and commit seals.
- `trustedHeight`: must use `revisionNumber == 0`.
- `accountProof`: raw RLP-encoded Ethereum account proof for the tracked `ICS26Router` account against the submitted header's `stateRoot`.

On update, the contract:

1. parses and validates the Besu header,
2. reconstructs the protocol-specific commit-seal digest following the YUI + prover sealing-header model,
3. checks trusted-validator overlap and new-validator quorum,
4. verifies the tracked router account proof,
5. stores the router account `storageRoot` plus the new validator set.

## Membership / non-membership proofs

`verifyMembership` and `verifyNonMembership` expect the standard `ILightClientMsgs` payloads used by Eureka.

For Besu / EVM counterparties, the expected merkle prefix is:

```solidity
[bytes("")]
```

That means `msg_.path[0]` is the raw Eureka commitment path bytes.

### Storage slot derivation

The counterparty `ICS26Router` stores commitments in `IBCStoreUpgradeable` as:

```solidity
mapping(bytes32 hashedPath => bytes32 commitment) commitments;
```

The proof key is derived as:

```solidity
bytes32 hashedPath = keccak256(rawPath);
bytes32 storageSlot = keccak256(abi.encode(hashedPath, IBCSTORE_STORAGE_SLOT));
```

where `IBCSTORE_STORAGE_SLOT` is the ERC-7201 namespace constant used by `IBCStoreUpgradeable`.

### Membership

- `msg_.proof` must be the raw RLP-encoded Ethereum storage proof for `storageSlot`.
- `msg_.value` must be exactly `abi.encodePacked(bytes32Commitment)`.
- The return value is the trusted consensus timestamp in seconds for `msg_.proofHeight`.

### Non-membership

- `msg_.proof` must be the raw RLP-encoded Ethereum storage proof for `storageSlot`.
- The proof must show that the slot is absent.
- The return value is the trusted consensus timestamp in seconds for `msg_.proofHeight`.

## Test fixtures

The Foundry fixtures under `test/besu-bft/fixtures/` can be regenerated from the focused Besu↔Besu e2e flow:

```sh
GENERATE_BESU_LIGHT_CLIENT_FIXTURES=true \
just test-e2e TestWithBesuToBesuTestSuite/Test_ICS20TransferERC20FromChainAToChainB
```

This writes `test/besu-bft/fixtures/qbft.json` using live Besu QBFT headers, account proofs, and storage proofs captured during the e2e transfer flow. The negative cases in that fixture are still derived by deterministic off-chain header mutation so the contract tests can keep explicit overlap / quorum / conflict coverage.

`ibft2.json` remains synthetic until an IBFT2-focused e2e fixture path is added.
