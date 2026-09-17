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
    bytes32 stateRoot;
    address[] validators;
}
```

The full consensus state is **not** retrievable from the contract; `getConsensusStateHash(uint64)` on `IBesuLightClient` returns only the stored hash and reverts with `ConsensusStateNotFound` for unknown heights. Every `updateClient`, `verifyMembership`, and `verifyNonMembership` call must carry the preimage of the consensus state it relies on, and the contract checks it against the stored hash before use. A mismatch reverts with `ConsensusStatePreimageMismatch(expectedHash, actualHash)`; an unknown height reverts with `ConsensusStateNotFound(height)`.

Relayers therefore need to keep the preimages of the heights they intend to reference, or rebuild them from the Besu chain: `timestamp`, `stateRoot`, and `validators` all come from the header at that height. Rebuilding a preimage never requires `eth_getProof`, so it keeps working for heights whose state a Bonsai-backed Besu node has already pruned.

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

## Destination EVM requirements

The Besu light clients are deployed on the **destination** EVM chain, next to that chain's `ICS26Router`. That chain must have the **Cancun** hard fork enabled, in particular [EIP-1153](https://eips.ethereum.org/EIPS/eip-1153) transient storage (`TSTORE` / `TLOAD`):

- `BesuLightClientBase` caches each proven router storage root in transient storage (see "Membership / non-membership proofs" below). On a chain without EIP-1153, every `verifyMembership` and `verifyNonMembership` call reverts with an invalid-opcode error when it tries to write that cache, so packet proof verification fails even when the proof itself is valid.
- This is not a new requirement relative to the rest of the stack: `ICS26Router`, `ICS20Transfer`, and `ICS27GMP` already use OpenZeppelin's `ReentrancyGuardTransient`, and `SP1ICS07Tendermint` caches proofs in transient storage, so a chain that can run the router can run these clients.
- All contracts in `ibc-solidity/` are compiled with `evm_version = "cancun"` (`ibc-solidity/foundry.toml`), so the compiled artifacts may also rely on other Shanghai and Cancun opcodes such as `PUSH0` and `MCOPY`. Do not lower `evm_version` to target an older chain; the transient cache has no fallback.

Before deploying to a new destination network, confirm that it reports Cancun as active. A quick check is to `eth_call` a probe that executes `TSTORE`; a pre-Cancun chain returns an invalid-opcode failure. The end-to-end suites exercise this on a Cancun target: Foundry tests run under the `cancun` EVM, and the Besu QBFT e2e genesis enables it with `"cancunTime": 0` (`e2e/interchaintestv8/chainconfig/testdata/besu/qbft/genesis.json`).

The **source** Besu chain, whose headers and proofs are being verified, has no hard-fork requirement beyond what `eth_getProof` needs; only the chain hosting the light client contract must support Cancun.

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
    bytes32 initialTrustedStateRoot,
    address[] memory initialTrustedValidators,
    uint64 trustingPeriod,
    uint64 maxClockDrift,
    address roleManager
)
```

- `ibcRouter`: counterparty `ICS26Router` proxy address whose account/storage proofs are tracked.
- `initialTrustedHeight`: trusted Besu block number. Revision number is always `0`.
- `initialTrustedTimestamp`: trusted header timestamp in seconds.
- `initialTrustedStateRoot`: state root of the Besu header at `initialTrustedHeight`.
- `initialTrustedValidators`: validator set trusted at `initialTrustedHeight`.
- `trustingPeriod`: weak-subjectivity window in seconds. Must be non-zero.
- `maxClockDrift`: allowed future drift for submitted headers in seconds.
- `roleManager`: if non-zero, receives admin and `PROOF_SUBMITTER_ROLE`; if zero, proof submission is open to anyone through the zero-address sentinel.

## `updateClient(bytes)` ABI

`updateClient` expects `abi.encode(IBesuLightClientMsgs.MsgUpdateClient)`:

```solidity
struct MsgUpdateClient {
    bytes headerRlp;
    IICS02ClientMsgs.Height trustedHeight;
    ConsensusState consensusStatePreimage;
}
```

- `headerRlp`: full raw Besu block header RLP, including `extraData` and commit seals.
- `trustedHeight`: must use `revisionNumber == 0` and identify a stored consensus state hash.
- `consensusStatePreimage`: the consensus state trusted at `trustedHeight`. Its hash must match the stored hash.

On update, the contract:

1. parses and validates the Besu header,
2. checks the trusted consensus state preimage against the stored hash and the trusting period,
3. reconstructs the protocol-specific commit-seal digest following the YUI + prover sealing-header model,
4. checks trusted-validator overlap against the preimage validators and quorum against the new header validators,
5. stores `keccak256(abi.encode(ConsensusState))` for the new height, built from the header timestamp, the header `stateRoot`, and the header validator set.

Submitting a header whose derived consensus state hash already matches the stored hash at that height returns `UpdateResult.NoOp`. A different consensus state at an already stored height reverts with `ConflictingConsensusState`.

## Membership / non-membership proofs

`verifyMembership` and `verifyNonMembership` expect the standard `ILightClientMsgs` payloads used by Eureka, with `msg_.proof` set to `abi.encode(IBesuLightClientMsgs.MembershipProof)`:

```solidity
struct MembershipProof {
    ConsensusState consensusStatePreimage;
    bytes[] accountProofNodes;
    bytes[] proofNodes;
}
```

- `consensusStatePreimage`: the consensus state trusted at `msg_.proofHeight`. Its hash must match the stored hash.
- `accountProofNodes`: the ordered, RLP-encoded Ethereum state-trie nodes proving the tracked `ICS26Router` account, as returned by `eth_getProof` at `msg_.proofHeight`. May be empty to reuse a storage root that an earlier call in the same transaction already proved for `msg_.proofHeight`; if none was cached, the call reverts with `StorageRootNotInCache(height)`.
- `proofNodes`: the ordered, RLP-encoded Ethereum storage-trie nodes for `storageSlot` as returned by `eth_getProof` at `msg_.proofHeight`.

Both calls first verify `accountProofNodes` against the preimage `stateRoot` to recover the router account's storage root, then verify `proofNodes` against that storage root. An account proof that does not resolve under the preimage `stateRoot` reverts with a `TrieProof.TrieProofTraversalError`. A successfully proven storage root is cached in transient storage, keyed by router address and height, so a batch of packet proofs against the same height only pays for one account proof: the first call carries `accountProofNodes` and the rest leave it empty. Supplying a non-empty account proof always verifies it, regardless of the cache.

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

- `proofNodes` must prove the commitment value under the router storage root recovered from `accountProofNodes`.
- `msg_.value` must be exactly `abi.encodePacked(bytes32Commitment)`.
- The return value is the trusted consensus timestamp in seconds for `msg_.proofHeight`, taken from the verified preimage.

### Non-membership

- `msg_.path` must contain exactly one element: the raw Eureka commitment path.
- `proofNodes` must establish that `storageKey` is absent from the router storage root recovered from `accountProofNodes`. Accepted exclusion witnesses include an empty trie (encoded as an empty `bytes[]`), an empty branch child or value, and a leaf or extension path that diverges from the derived key.
- The proof must end at the node that establishes exclusion; extra trailing proof nodes are rejected.
- A valid exclusion proof returns the trusted consensus timestamp in seconds for `msg_.proofHeight`, taken from the verified preimage. If the decoded trie proof does not establish exclusion for the trusted root and derived key, including when it proves an existing value, the call reverts with `InvalidExclusionProof`. Malformed ABI or RLP data may revert while being decoded.

This verification supports packet timeout flows that prove the absence of a packet receipt on a Besu counterparty.

## Generated Go payload types

`scripts/IBesuLightClientEncoding.sol` is a generation-only interface referencing the
`IBesuLightClientMsgs` structs. It exposes the tuple schemas that are hidden inside
the production light client's `bytes` inputs and outputs; it is never deployed.

Run `just solidity::generate-abi` from the repository root to regenerate the Go bindings.

Create `besumsgs.NewBindings()` and use its generated Go structs and `Pack*` or
`TryPack*` methods. These methods encode function calls, including a four-byte
selector. **Remove the first four bytes** to obtain the `abi.encode(value)` payload
expected by the light client:

```go
bindings := besumsgs.NewBindings()
encoded, err := bindings.TryPackProofNodes(nodes)
if err != nil {
    return nil, err
}
return encoded[4:], nil
```

The consensus state commitment is
`crypto.Keccak256Hash(bindings.PackConsensusState(state)[4:])`.
`Pack*` panics on invalid inputs; `TryPack*` returns an error. Callers retain
light-client policy checks. Matching schema return types generate typed `Unpack*`
helpers for decoding the same `abi.encode(value)` payloads, without a selector.

## Test fixtures

The Foundry fixtures under `test/besu-bft/fixtures/` can be regenerated from the focused Besu↔Besu e2e flow:

```sh
just solidity::generate-fixtures-besu
```

This writes `test/besu-bft/fixtures/qbft.json` using live Besu QBFT headers, account proofs, and storage proofs captured during the e2e transfer flow. The fixture `proof` and `accountProof` fields hold the raw storage and account proof nodes as `abi.encode(bytes[])`; the Foundry tests wrap them into `MembershipProof` together with the consensus state preimage derived from the fixture's expected update state. The negative cases in that fixture are still derived by deterministic off-chain header mutation so the contract tests can keep explicit overlap / quorum / conflict coverage.

The synthetic IBFT2 validator sets and commit seals, and QBFT's synthetic low-overlap
case, can be regenerated offline with the existing Go header and signing helpers:

```sh
cd e2e/interchaintestv8
go test ./types -run '^TestBesu(IBFT2Fixture|QBFTLowOverlapFixture)$' -args -update-besu-synthetic
```

Run this in the Nix development shell. Omitting the update flag checks that the
fixtures match the generators. Regeneration preserves the other header fields and
trie proofs. `ibft2.json` remains synthetic until an IBFT2-focused e2e fixture path is added.
