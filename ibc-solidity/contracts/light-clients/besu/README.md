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
- `trustingPeriod`: weak-subjectivity window in seconds. `0` disables expiry
  and is unsuitable for production non-adjacent verification.
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
- `trustedHeight`: must use `revisionNumber == 0` and be lower than the submitted header height.
- `accountProof`: raw RLP-encoded Ethereum account proof for the tracked `ICS26Router` account against the submitted header's `stateRoot`.

On update, the contract:

1. parses the full signed header and validates the fields required by this non-adjacent verification model,
2. reconstructs the protocol-specific commit-seal digest following the YUI + prover sealing-header model,
3. checks trusted-validator overlap and new-validator quorum,
4. verifies the tracked router account proof,
5. stores the router account `storageRoot` plus the new validator set.

Non-adjacent updates are supported by design. The submitted height must be
strictly after the selected trusted height. If validator turnover prevents the
required overlap, the relayer must first submit one or more intermediate
headers; they do not need to be adjacent.

The contract authenticates the complete header through its commit seals and
validates the fields used by this verifier. It does not re-execute source-chain
transactions; those semantics remain covered by the honest-validator
assumption of the trusting-period model.

The parser accepts the standard 15 Ethereum header fields plus signed
fork-extension fields. Deployments must confirm that the configured Besu fork
keeps the standard fields at their expected indices.

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

### Trie proof implementation

`MPTProof.sol` is a narrow fork of OpenZeppelin Contracts v5.6.1
[`TrieProof.sol`](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.6.1/contracts/utils/cryptography/TrieProof.sol).
The upstream library verifies inclusion only and its private traversal helpers
cannot be wrapped or inherited to add exclusion.

The fork keeps OpenZeppelin's node-link validation, compact-path traversal, and
inline-node handling. Its local changes are limited to:

1. decoding this client's existing RLP-encoded proof-list wire format,
2. returning an explicit `exists` flag,
3. accepting authenticated exclusion at an empty trie, empty branch child or
   value, or divergent compact leaf/extension path, and
4. rejecting proof nodes supplied after any inclusion or exclusion terminal.

When updating OpenZeppelin, diff `MPTProof.sol` against the pinned upstream file
above and run `forge test --match-path "test/besu-bft/*.sol"`.

## Test fixtures

The Foundry fixtures under `test/besu-bft/fixtures/` can be regenerated from the focused Besu↔Besu e2e flow:

```sh
GENERATE_BESU_LIGHT_CLIENT_FIXTURES=true \
just test-e2e TestWithBesuToBesuTestSuite/Test_ICS20TransferERC20FromChainAToChainB
```

This writes `test/besu-bft/fixtures/qbft.json` using live Besu QBFT headers, account proofs, and storage proofs captured during the e2e transfer flow. The negative cases in that fixture are still derived by deterministic off-chain header mutation so the contract tests can keep explicit overlap / quorum / conflict coverage.

`ibft2.json` remains synthetic until an IBFT2-focused e2e fixture path is added.
