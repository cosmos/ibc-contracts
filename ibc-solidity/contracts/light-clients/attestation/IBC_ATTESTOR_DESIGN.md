## Solidity IBC Attestor Light Client — Design

This document specifies how the Solidity light client for IBC Attestations works. It mirrors the high-level semantics of the CosmWasm 08-wasm attestor client while adapting details to EVM constraints and `ILightClient` in this repository. It focuses on behavior and interfaces, not implementation.

### Goals
- Provide an IBC light client that trusts an off-chain m-of-n attestor set for state updates and packet membership verification.
- Be compatible with the existing attestor/aggregator services and the Rust core logic where possible.
- Integrate with `ICS26Router` via `ILightClient`.
- Support membership checks for packet commitments attested as a list of commitments.

### Non-goals (initial scope)
- Misbehaviour evidence processing and freezing (placeholder/TODO only).
- Time validity windows (no clock-drift checks beyond stored timestamps).

## System Overview

- Off-chain attestors sign opaque attestation data. The aggregator collects signatures and forms headers/proofs.
- On-chain, the light client:
  - Verifies m-of-n signatures over the attested data using the configured attestor set.
  - Updates a minimal consensus state per height: `(height, timestamp)`.
  - Verifies membership by checking that a provided packet commitment is contained in an attested list for a given height.

## Data Model

- ClientState
  - `attestorAddresses: address[]` — fixed set of attestor identities (addresses favored on EVM for verification efficiency).
  - `minRequiredSigs: uint8` — quorum threshold.
  - `latestHeight: uint64` — highest known height.
  - `isFrozen: bool` — reserved for misbehaviour (not used until TODO is implemented).

- Consensus state storage
  - `mapping(uint64 => uint64) consensusTimestampAtHeight` — maps `height -> timestampSeconds`.
  - Storing height and timestamp is sufficient. No commitment root is stored.

## Hashing and Signatures

- Hashing: SHA-256 of the exact `attestationData` bytes (same as Rust core). On-chain we use Solidity `sha256(bytes)` to produce the 32-byte digest used for signature checks.
- Signatures: secp256k1 ECDSA signatures with 65 bytes `(r || s || v)` per signature.
  - We use OpenZeppelin `ECDSA.recover(digest, signature)` for recovery and low-s malleability protection. `ECDSA.recover` accepts `v` in either 27/28 or 0/1 form; we will accept both.
  - Each recovered signer address must belong to the fixed attestor set, and each must be unique within the verification set. Enforce `minRequiredSigs` quorum.
  - Each signature MUST be 65 bytes. Unknown or duplicate signers, or failing to meet the threshold, cause reverts.

## Encodings and Wire Compatibility

- Header/proof payloads passed to the Solidity client are ABI-encoded for gas efficiency and simplicity.
- Attested data must be decoded on-chain, so JSON/Serde payloads used by Rust are not practical on EVM.
- Proof envelope used by the client:
  - `AttestationProof { attestationData: bytes, signatures: bytes[] }`
  - `signatures[i]` is a 65-byte `(r||s||v)` over `sha256(attestationData)`.
- The `attestationData` payload is one of the following ABI-encoded structs, depending on the operation:
  - `StateAttestation { uint64 height; uint64 timestamp; }` — used by `updateClient`.
  - `PacketAttestation { uint64 height; PacketCompact[] packets; }` — used by `verifyMembership`.
- Public keys are not propagated anywhere on chain; addresses are inferred by recovering from signatures.
- This enables efficient on-chain decoding and membership checks.
- TODO: Update/align the CosmWasm attestor client to use ABI-equivalent payloads so both clients are wire-compatible at the attested-data level.

## External Interface (behavioral)

The client implements `ILightClient` with the following behaviors.

### updateClient(updateMsg)

Purpose: Validate an aggregated attestation signed by the attestor set and update the consensus state at the given height.

Input (ABI-encoded inside `updateMsg`):
- `AttestationProof` where `attestationData = abi.encode(StateAttestation{height, timestamp})` and `signatures` is an array of 65-byte `(r||s||v)`.

Verification:
- Compute `digest = sha256(proof.attestationData)`.
- For each `signature65` use `ECDSA.recover(digest, signature65)` to obtain `signer`.
- Require each `signer` to be in `attestorAddresses` and all recovered signers to be unique.
- Enforce `numUniqueValidSigners >= minRequiredSigs`.
- Decode `proof.attestationData` into `StateAttestation { height, timestamp }`.

State transition:
- If `consensusTimestampAtHeight[height]` already exists:
  - If `timestamp == consensusTimestampAtHeight[height]`: return `UpdateResult.NoOp` (idempotent).
  - Else: freeze the client and return `UpdateResult.Misbehaviour`.
- Else (first time for this height):
  - Set `consensusTimestampAtHeight[height] = timestamp`.
  - Set `latestHeight = height`.
  - Return `UpdateResult.Update`.

Access control:
- Only callable by `ICS26_ROUTER`, the router the client was deployed for (same semantics as `SP1ICS07Tendermint`). Relayers submit updates through the router's `updateClient`.
- Reverts while the client is paused.

Notes:
- No time-window or clock-drift validation is performed.
- Monotonic height increases are NOT enforced in the current version; out-of-order updates are accepted. `latestHeight` is set to the attested `height`, even if it decreases. See TODOs.

### verifyMembership(MsgVerifyMembership)

Purpose: Verify that a single packet commitment value exists within an attested list at a given height and return the trusted timestamp for that height.

Usage:
- `value` is a single packet commitment `bytes` (ABI-encoded `bytes32`).
- The attested list and height are passed inside the `proof` and must be the exact data signed by the attestors.

Input mapping (from `ILightClientMsgs.MsgVerifyMembership`):
- `proof: bytes` — `abi.encode(AttestationProof)` where `attestationData = abi.encode(PacketAttestation{height, packets})`.
- `proofHeight: Height` — the height to verify against.
- `path: bytes[]` — The path of the commitment.
- `value: bytes` — ABI-encoded `bytes32` packet commitment to check for membership.

Verification:
- Require `consensusTimestampAtHeight[proofHeight.revisionHeight]` to exist.
- Compute `digest = sha256(proof.attestationData)` and verify signatures as in `updateClient` via `ECDSA.recover`.
- Decode `attestationData` into `PacketAttestation { height, packets }` via `abi.decode` and require `height == proofHeight.revisionHeight`.
- Decode `pathHash` to `bytes32` and require it to match one `sha256(path[i])` in the `packets` array.
- Decode `value` to `bytes32` and require it to match the corresponding `packet.commitment` in the matched `packets` entry.

Return:
- The trusted timestamp (in seconds) stored for `proofHeight.revisionHeight`.

Access control:
- Only callable by `ICS26_ROUTER`. Relayers can verify proofs ahead of packet handling through the router's restricted routing function of the same name.
- Reverts while the client is paused.

### verifyNonMembership(MsgVerifyNonMembership)

Purpose: Verify non-membership of a packet commitment in the counterparty chain's state at a given height and return the trusted timestamp for that height.

Usage:
- `value` in the attestation is `bytes32(0)` to indicate non-membership.
- The attested list and height are passed inside the `proof` and must be the exact data signed by the attestors.

Input mapping (from `ILightClientMsgs.MsgVerifyNonMembership`):

- `proof: bytes` — `abi.encode(AttestationProof)` where `attestationData = abi.encode(PacketAttestation{height, packets})`.
- `proofHeight: Height` — the height to verify against.
- `path: bytes[]` — The path of the non-membership check.

Verification:
- Require `consensusTimestampAtHeight[proofHeight.revisionHeight]` to exist.
- Compute `digest = sha256(proof.attestationData)` and verify signatures as in `updateClient` via `ECDSA.recover`.
- Decode `attestationData` into `PacketAttestation { height, packets }` via `abi.decode` and require `height == proofHeight.revisionHeight`.
- Decode `pathHash` to `bytes32` and require it to match one of the SHA-256 hashes of the corresponding `path` entries in `packets`.
- Decode `value` to `bytes32` and require it to match exactly `bytes32(0)` to indicate non-membership.

Return:
- The trusted timestamp (in seconds) stored for `proofHeight.revisionHeight`.

Access control:
- Only callable by `ICS26_ROUTER`. Relayers can verify proofs ahead of packet handling through the router's restricted routing function of the same name.
- Reverts while the client is paused.

### misbehaviour(...)

Out of scope for this version. The function exists but MUST revert with a clear TODO note to implement evidence verification and freezing.

## Roles and Permissions

The client has no roles of its own. It is bound at construction to an immutable, non-zero `ICS26_ROUTER`:

- `updateClient`, `verifyMembership`, `verifyNonMembership` and `misbehaviour` can only be called by `ICS26_ROUTER`.
- `pause()` and `unpause()` can be called by accounts with `IBCRolesLib.PAUSER_ROLE` and `IBCRolesLib.UNPAUSER_ROLE`, without an execution delay, in the router's current `AccessManager` (`ICS26_ROUTER.authority()`). While paused, `updateClient`, `verifyMembership` and `verifyNonMembership` revert.

## Time and Ordering

- No validity window or clock-drift checks are enforced. The timestamp stored in consensus state is treated as an attested fact tied to the height.
- Height progression is not enforced to be monotonic in this version. Replays at the same height with identical timestamp are idempotent (`NoOp`). Conflicts at the same height cause a revert with `ConflictingTimestamp`.

## Replay and Domain Separation

This version does not embed domain separation fields in the signed attestation bytes. This introduces potential replay risk across different clients or contexts if the same `attestationData` can be reused elsewhere.

> [!WARNING]
>
> Same attestor addresses must never be re-used for multiple clients or contexts.

Recommended future hardening (non-breaking if added to the signed payload definition):
- Include a typed domain in the attested bytes such as one or more of:
  - counterparty chain-id, IBC `client-id`, this light client contract address, local chain-id, and a protocol tag/version.
- Document exact encoding and update both Solidity and CosmWasm clients accordingly.

## Security Considerations

- Threshold enforcement: enforce uniqueness of recovered signer addresses and minimum quorum.
- Signature malleability: OpenZeppelin `ECDSA` enforces low-`s` and rejects malleable signatures.
- Unknown signers and duplicate signers cause reverts; each signature must be exactly 65 bytes.
- Attestor set management: fixed in client state for initial scope; rotation requires a future governance-controlled upgrade path.
- Large attested lists: no explicit limits are imposed in this design; callers must size proofs responsibly to avoid out-of-gas.

## ICS-26 Integration

- The client implements `ILightClient`. The router invokes:
  - `updateClient` to set or confirm consensus timestamps by height.
  - `verifyMembership` during proof verification flows (returns the counterparty timestamp in seconds).
  - Other interface methods revert as described above.

## Open TODOs / Future Work
- Implement misbehaviour detection and freezing (conflicting attestations for the same height, etc.).
- Consider adding domain separation to the signed payload.
- Implement monotonic increase check for `latestHeight` and/or restrict out-of-order updates.
- Consider attestor set rotation and quorum changes.
- Optional: proof caching within a transaction if needed for multi-membership checks.
