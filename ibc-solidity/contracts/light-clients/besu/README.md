# Besu IBFT 2.0 / QBFT Light Clients

This module contains two Solidity light clients for Besu BFT chains:

- `BesuIBFT2LightClient.sol`
- `BesuQBFTLightClient.sol`

Both wrappers share the same storage model and proof surface through `BesuLightClientBase.sol`. They differ only in how the commit-seal signing digest is reconstructed from the raw Besu header.

## Storage model

The client stores the `ClientState` and, per trusted height, only the `keccak256(abi.encode(ConsensusState))` hash:

```solidity
struct ConsensusState {
    uint64 timestamp;
    bytes32 storageRoot;
    address[] validators;
}
```

The full consensus state is **not** retrievable from the contract. Every `updateClient`, `verifyMembership`, and `verifyNonMembership` call must carry the preimage of the consensus state it relies on, and the contract checks it against the stored hash before use. A mismatch reverts with `ConsensusStatePreimageMismatch(expectedHash, actualHash)`; an unknown height reverts with `ConsensusStateNotFound(height)`.

Relayers therefore need to keep the preimages of the heights they intend to reference, or rebuild them from the Besu chain: `timestamp` and `validators` come from the header at that height, and `storageRoot` is the tracked router account's storage hash from `eth_getProof` at that height.

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
    ConsensusState consensusStatePreimage;
    bytes accountProof;
}
```

- `headerRlp`: full raw Besu block header RLP, including `extraData` and commit seals.
- `trustedHeight`: must use `revisionNumber == 0` and identify a stored consensus state hash.
- `consensusStatePreimage`: the consensus state trusted at `trustedHeight`. Its hash must match the stored hash.
- `accountProof`: Ethereum account proof nodes for the tracked `ICS26Router` account, encoded as `abi.encode(bytes[])`.

On update, the contract:

1. parses and validates the Besu header,
2. checks the trusted consensus state preimage against the stored hash and the trusting period,
3. reconstructs the protocol-specific commit-seal digest following the YUI + prover sealing-header model,
4. checks trusted-validator overlap against the preimage validators and quorum against the new header validators,
5. verifies the tracked router account proof,
6. stores `keccak256(abi.encode(ConsensusState))` for the new height, built from the header timestamp, the proven router `storageRoot`, and the header validator set.

Submitting a header whose derived consensus state hash already matches the stored hash at that height returns `UpdateResult.NoOp`. A different consensus state at an already stored height reverts with `ConflictingConsensusState`.

## Membership / non-membership proofs

`verifyMembership` and `verifyNonMembership` expect the standard `ILightClientMsgs` payloads used by Eureka, with `msg_.proof` set to `abi.encode(IBesuLightClientMsgs.MembershipProof)`:

```solidity
struct MembershipProof {
    ConsensusState consensusStatePreimage;
    bytes[] proofNodes;
}
```

- `consensusStatePreimage`: the consensus state trusted at `msg_.proofHeight`. Its hash must match the stored hash.
- `proofNodes`: the ordered, RLP-encoded Ethereum storage-trie nodes for `storageSlot` as returned by `eth_getProof` at `msg_.proofHeight`.

`msg_.proofHeight` must use revision number `0` and identify a stored consensus state hash.

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
bytes memory storageKey = abi.encodePacked(keccak256(abi.encodePacked(storageSlot)));
```

where `IBCSTORE_STORAGE_SLOT` is the ERC-7201 namespace constant used by `IBCStoreUpgradeable`.

### Membership

- `proofNodes` must prove the commitment value under the preimage `storageRoot`.
- `msg_.value` must be exactly `abi.encodePacked(bytes32Commitment)`.
- The return value is the trusted consensus timestamp in seconds for `msg_.proofHeight`, taken from the verified preimage.

### Non-membership

- `msg_.path` must contain exactly one element: the raw Eureka commitment path.
- `proofNodes` must establish that `storageKey` is absent from the preimage `storageRoot`. Accepted exclusion witnesses include an empty trie (encoded as an empty `bytes[]`), an empty branch child or value, and a leaf or extension path that diverges from the derived key.
- The proof must end at the node that establishes exclusion; extra trailing proof nodes are rejected.
- A valid exclusion proof returns the trusted consensus timestamp in seconds for `msg_.proofHeight`, taken from the verified preimage. If the decoded trie proof does not establish exclusion for the trusted root and derived key, including when it proves an existing value, the call reverts with `InvalidExclusionProof`. Malformed ABI or RLP data may revert while being decoded.

This verification supports packet timeout flows that prove the absence of a packet receipt on a Besu counterparty.

## Test fixtures

The Foundry fixtures under `test/besu-bft/fixtures/` can be regenerated from the focused Besu↔Besu e2e flow:

```sh
just solidity::generate-fixtures-besu
```

This writes `test/besu-bft/fixtures/qbft.json` using live Besu QBFT headers, account proofs, and storage proofs captured during the e2e transfer flow. The fixture `proof` fields hold the raw storage proof nodes as `abi.encode(bytes[])`; the Foundry tests wrap them into `MembershipProof` together with the consensus state preimage derived from the fixture's expected update state. The negative cases in that fixture are still derived by deterministic off-chain header mutation so the contract tests can keep explicit overlap / quorum / conflict coverage.

`ibft2.json` remains synthetic until an IBFT2-focused e2e fixture path is added.
