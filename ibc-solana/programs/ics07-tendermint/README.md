<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ICS07 Tendermint Light Client for Solana

This is a Solana implementation of the ICS07 Tendermint light client, enabling IBC (Inter-Blockchain Communication) between Solana and Tendermint-based chains.

## Overview

The Tendermint light client verifies consensus proofs from Tendermint chains on Solana. This implementation supports multiple independent light client instances, each tracking a different Tendermint-based chain. After initialization, each client instance is identified by its chain ID and can be used to verify proofs from its corresponding chain.

Since Tendermint headers are always several KB in size and exceed Solana's transaction size limitations (~1232 bytes), all headers must be uploaded in chunks across multiple transactions before being assembled and verified. Each chunk can contain up to 900 bytes of header data.

## Architecture

### Chunked Upload System

The client implements a mandatory chunked upload mechanism for all header updates:

1. **Headers are split into chunks**: Large headers are divided into manageable pieces that fit within Solana transactions
2. **Metadata creation is separate**: The `create_metadata` instruction initializes the upload metadata once per upload
3. **Full parallel upload support**: All chunks can be uploaded in parallel after metadata creation for maximum throughput
4. **Each relayer has isolated storage**: Chunks are stored in PDAs derived from the relayer's address, preventing interference between relayers
5. **Relayers pay rent**: The submitting relayer pays for all storage (metadata and chunk accounts)
6. **Assembly triggers verification**: Once all chunks are uploaded, the relayer calls `assemble_and_update_client` to reconstruct and verify the header

### Key Design Principles

- **Relayer Isolation**: Each relayer's uploads are completely isolated via PDAs keyed by submitter address
- **Rent Economics**: Relayers are incentivized to clean up failed uploads as they pay the rent
- **Atomic Updates**: Header assembly and client updates happen atomically - either all succeed or all fail
- **Automatic Cleanup**: Successful assembly automatically closes temporary accounts and returns rent

## Program Instructions

### Core IBC Instructions

#### `initialize`

Initializes a new Tendermint light client instance for a specific chain. Multiple clients can be initialized to track different Tendermint-based chains simultaneously.

**Parameters:**

- `chain_id`: The unique chain identifier (e.g., "cosmoshub-4", "osmosis-1", "noble-1")
- `latest_height`: Initial trusted height
- `client_state`: Initial client configuration (trust level, periods, etc.)
- `consensus_state`: Initial trusted consensus state
- `access_manager`: The access manager program ID for role-based access control

**Accounts:**

- `client_state` (init): PDA storing client configuration, derived from chain_id
- `consensus_state_store` (init): PDA storing consensus state at height
- `app_state` (init): PDA storing global application state including access manager
- `payer` (signer, mut): Account paying for initialization
- `system_program`: System program

**Multi-Chain Support**: Each chain_id creates a separate client instance with its own state. This allows Solana to maintain IBC connections with multiple Tendermint chains concurrently.

#### `propose_access_manager_transfer`

Starts a two-step transfer to a new access manager program ID.
Requires `ADMIN_ROLE` on the current access manager.

**Parameters:**

- `new_access_manager`: Proposed new access manager program ID

**Accounts:**

- `app_state` (mut): PDA storing global application state
- `access_manager`: Current access manager state PDA used for role validation
- `admin` (signer): Admin on the current access manager
- `instructions_sysvar`: Instructions sysvar for CPI validation

#### `accept_access_manager_transfer`

Finalizes a pending transfer and switches `app_state.am_state.access_manager` to the proposed program ID.
Requires `ADMIN_ROLE` on the new (pending) access manager.

**Parameters:**

- None

**Accounts:**

- `app_state` (mut): PDA storing global application state, must have a pending transfer
- `new_am_state`: New access manager state PDA derived from `pending_access_manager`
- `admin` (signer): Admin on the new access manager
- `instructions_sysvar`: Instructions sysvar for CPI validation

#### `cancel_access_manager_transfer`

Cancels a pending transfer.
Requires `ADMIN_ROLE` on the current access manager.

**Parameters:**

- None

**Accounts:**

- `app_state` (mut): PDA storing global application state
- `am_state`: Current access manager state PDA used for role validation
- `admin` (signer): Admin on the current access manager
- `instructions_sysvar`: Instructions sysvar for CPI validation

For AM-A -> AM-B migration context shared across IBC programs, see [AccessManager](../access-manager/README.md).

### Chunked Upload Instructions

Since Tendermint headers always exceed Solana's transaction size limits, all header updates must use the chunked upload system described below.

#### `upload_header_chunk`

Uploads a single chunk of a large header. Each chunk creates a new PDA.

**Parameters:**

- `params`: UploadChunkParams containing:
  - `chain_id`: Target chain
  - `target_height`: Height being updated to
  - `chunk_index`: Position of this chunk (0-indexed)
  - `chunk_data`: The actual chunk bytes (max 900 bytes)

**Accounts:**

- `chunk` (init): PDA for this specific chunk (fails if already exists)
- `client_state`: Validates chain exists and chain_id matches
- `submitter` (signer, mut): Relayer uploading and paying rent
- `system_program`: System program

**Storage Cost**: Each chunk account costs rent, paid by submitter

**Parallel Upload**: All chunks can be uploaded in parallel transactions for faster throughput. Each chunk upload is independent.

#### `assemble_and_update_client`

Assembles uploaded chunks into a complete header and updates the client.

**Parameters:**

- `chain_id`: Target chain identifier
- `target_height`: Height being updated to
- `chunk_count`: Number of chunks to assemble

**Accounts:**

- `client_state` (mut): Client being updated
- `app_state`: PDA storing global application state
- `access_manager`: Access manager account for role validation
- `trusted_consensus_state`: Consensus at trusted height (unchecked, validated in handler)
- `new_consensus_state_store`: New consensus state account (unchecked, may not exist yet)
- `submitter` (signer, mut): Relayer submitting the update
- `system_program`: System program
- `instructions_sysvar`: Instructions sysvar for CPI validation
- Remaining accounts: Chunk accounts in order

**Process:**

1. Validates all chunks are present
2. Reconstructs complete header from chunks
3. Verifies header against trusted state
4. Updates client state

**Note:** Chunk accounts are NOT automatically closed. Relayers must call `cleanup_incomplete_upload` after success to reclaim rent.

#### `cleanup_incomplete_upload`

Allows relayers to reclaim rent from uploads. Must be called after both successful and failed uploads to close `HeaderChunk` and `SignatureVerification` PDAs owned by the submitter.

**Accounts:**

- `submitter` (signer, mut): Original submitter who gets their rent back
- Remaining accounts: Chunk and signature verification accounts to close

**Security**: Only the original submitter can clean up their own uploads

#### `pre_verify_signature`

Pre-verifies an Ed25519 signature using Solana's Ed25519Program precompile. This saves compute units during `assemble_and_update_client` by storing verification results in a PDA.

**Parameters:**

- `signature`: SignatureData containing the signature hash to verify

**Accounts:**

- `instructions_sysvar`: Instructions sysvar for Ed25519Program verification
- `signature_verification` (init): PDA storing the verification result
- `payer` (signer, mut): Account paying for the verification PDA
- `system_program`: System program

**Notes:**

- Multiple signatures can be pre-verified in parallel transactions
- Critical for large validator sets (50+ validators) that would exceed Solana's 1.4M CU limit with pure on-chain verification
- Verification PDAs are checked during `assemble_and_update_client` and can fall back to on-chain verification if not present

### Status Instructions

#### `client_status`

Returns the current status of the light client: `Active`, `Frozen`, or `Expired`. The router calls this via CPI before `send_packet` to ensure packets are only sent through healthy clients.

**Status determination:**

1. **Frozen** — if the client has been frozen due to misbehaviour detection
2. **Expired** — if the elapsed time since the latest consensus state timestamp exceeds the client's `trusting_period`
3. **Active** — otherwise

**Accounts:**

- `client_state`: Client configuration (includes `trusting_period` and `is_frozen` flag)
- `consensus_state`: Consensus state at `latest_height` (provides the timestamp for expiry calculation)

**Return data:** A single `u8` byte via `set_return_data`: `0` = Active, `1` = Frozen, `2` = Expired.

### Verification Instructions

#### `verify_membership`

Verifies a key-value pair exists in the counterparty chain's state.

**Parameters:**

- `msg`: MembershipMsg with proof details

**Accounts:**

- `client_state`: Client configuration
- `consensus_state_at_height`: Consensus state at proof height

#### `verify_non_membership`

Verifies a key does not exist in the counterparty chain's state.

**Parameters:**

- `msg`: MembershipMsg with proof details

**Accounts:**

- `client_state`: Client configuration
- `consensus_state_at_height`: Consensus state at proof height

### Misbehaviour Handling

Misbehaviour submission uses a chunked upload system similar to header updates, as misbehaviour proofs contain two headers and can also exceed transaction size limits.

#### `upload_misbehaviour_chunk`

Uploads a single chunk of misbehaviour data.

**Parameters:**

- `params`: UploadMisbehaviourChunkParams containing:
  - `client_id`: Target client identifier
  - `chunk_index`: Position of this chunk (0-indexed)
  - `chunk_data`: The actual chunk bytes (max 900 bytes)

**Accounts:**

- `chunk` (init): PDA for this specific misbehaviour chunk
- `client_state`: Validates client exists and client_id matches
- `submitter` (signer, mut): Relayer uploading and paying rent
- `system_program`: System program

#### `assemble_and_submit_misbehaviour`

Assembles uploaded misbehaviour chunks and freezes the client if valid.

**Parameters:**

- `client_id`: Target client identifier

**Accounts:**

- `client_state` (mut): Client to potentially freeze
- `app_state`: PDA storing global application state
- `access_manager`: Access manager account for role validation
- `trusted_consensus_state_1`: First trusted state
- `trusted_consensus_state_2`: Second trusted state
- `submitter` (signer, mut): Relayer submitting the misbehaviour
- `instructions_sysvar`: Instructions sysvar for CPI validation
- Remaining accounts: Misbehaviour chunk accounts in order

**Note:** Chunk accounts are NOT automatically closed. Relayers must call `cleanup_incomplete_misbehaviour` after success to reclaim rent.

#### `cleanup_incomplete_misbehaviour`

Allows relayers to reclaim rent from misbehaviour submissions. Must be called after both successful and failed submissions to close chunk accounts.

**Parameters:**

- `client_id`: Target client identifier
- `submitter`: Original submitter address

**Accounts:**

- `client_state`: Validates client exists
- `submitter_account` (signer, mut): Must be original submitter
- Remaining accounts: Misbehaviour chunk accounts to close

## PDA Derivations

All storage uses Program Derived Addresses (PDAs) for deterministic addressing. The chain_id is a key component in most PDAs, ensuring complete isolation between different chain clients:

```
app_state: [b"app_state"]
client_state: [b"client", chain_id]
consensus_state: [b"consensus_state", client_state, height_bytes]
header_chunk: [b"header_chunk", submitter, chain_id, height_bytes, chunk_index]
misbehaviour_chunk: [b"misbehaviour_chunk", submitter, client_id, chunk_index]
signature_verification: [b"sig_verify", signature_hash]
```

This PDA structure ensures that:

- Global app state manages access control
- Each chain has its own isolated client state
- Consensus states are chain-specific
- Upload operations cannot interfere across different chains
- Multiple chains can be tracked simultaneously without conflicts
- Signature pre-verification results are cached for reuse

## Upload Flow Example

```
1. Relayer receives 3.6KB Tendermint header
2. Splits into 4 chunks of 900 bytes each
3. Uploads all chunks in parallel (can be done in parallel):
   - Chunks 0-3: All uploaded independently in parallel
4. Optionally pre-verifies signatures in parallel (for large validator sets)
5. Calls assemble_and_update_client:
   - Header reconstructed and verified
   - Client state updated
6. Calls cleanup_incomplete_upload:
   - All 4 chunk accounts closed
   - Signature verification accounts closed
   - Rent (~0.05 SOL) returned to relayer
```

### Parallel Upload Optimization

For maximum throughput with large headers:

1. Upload all chunks in parallel transactions
2. Optionally pre-verify signatures in parallel (for 50+ validators)
3. Wait for all confirmations
4. Call `assemble_and_update_client`

This parallel approach can reduce upload time from `n * block_time` to `2 * block_time` for n chunks.

## Rent and Economics

- **Temporary Storage**: Chunks and signature verifications are temporary, existing only during upload
- **Rent Responsibility**: Uploading relayer pays all rent (~0.01 SOL per account)
- **Manual Cleanup Required**: Relayers must call cleanup instructions after both successful and failed operations to reclaim rent
- **No Cross-Relayer Interference**: Each relayer's uploads are isolated

## Security Considerations

1. **Permissioned Cleanup**: Only the original submitter can clean up their uploads
2. **Commitment Verification**: Header commitment prevents chunk tampering
3. **Height Validation**: Cannot upload chunks for already-processed heights
4. **Client Freezing**: Misbehaviour detection can freeze compromised clients
5. **Trusted Height Validation**: Updates must reference valid trusted heights

## Testing

The implementation includes comprehensive tests for:

- Happy path updates with real Tendermint fixtures
- Chunked upload and assembly
- Error conditions (missing chunks, wrong order, corruption)
- Misbehaviour detection
- Rent reclamation
- Multi-relayer scenarios

Run tests:

```bash
cargo test --package ics07-tendermint
```

## Gas/Compute Costs

Approximate compute units per operation:

- `initialize`: ~50k CU
- `upload_header_chunk`: ~30k CU per chunk
- `pre_verify_signature`: ~10k CU per signature (via Ed25519Program precompile)
- `assemble_and_update_client`: ~200k CU (includes verification)
- `verify_membership`: ~100k CU
- `verify_non_membership`: ~100k CU
- `upload_misbehaviour_chunk`: ~30k CU per chunk
- `assemble_and_submit_misbehaviour`: ~150k CU
- `cleanup_incomplete_upload`: ~20k CU per chunk
- `cleanup_incomplete_misbehaviour`: ~20k CU per chunk

### Update Client Performance Optimizations (Real-World Benchmarks)

The update client implementation includes several optimizations tested with real Tendermint chains (Noble: 20 validators, Celestia: 100 validators):

**Ed25519 Signature Verification:**

- **Pre-verification (optional):** Uses Ed25519Program precompile **~10k CU per signature** (via separate transaction)
  - Can be parallelized across multiple transactions for faster verification
  - **Critical for large validator sets (50+ validators):** Pure brine-ed25519 would exceed Solana's 1.4M CU transaction limit
  - Saves CU costs and enables support for validator sets of any size (given base assemble has enough CU for deserialization/verification of non-signature operations)
  - Pre-verification PDAs store validation results, checked during `assemble_and_update_client`
  - **Latency (real-world, RPC-dependent):**
    - Noble (20 validators, ~14 sigs at 2/3): ~18s total (Phase 1: ~3.3s for 19 prep txs, Phase 2: ~15s assembly)
    - Celestia (100 validators, ~67 sigs at 2/3): ~22s total (Phase 1: ~6s for 83 prep txs, Phase 2: ~16s assembly)
    - **Note:** Highly dependent on RPC throttling/rate limiting - implementation uses many parallel transactions
    - Optimal conditions (no RPC limits): Could complete within ~2 blocks (≈1s)
- **Fallback verification:** brine-ed25519 on-chain **~30k CU per signature**
  - Always available as fallback when pre-verification PDAs are not provided
  - Allows verification to succeed even without pre-computation step
  - Works well for smaller validator sets that fit within CU limits
- **Savings examples (2/3 threshold):**
  - Noble (20 validators, ~14 sigs): ~280k CU saved with pre-verification vs pure brine
  - Celestia (100 validators, ~67 sigs): ~1,340k CU saved with pre-verification vs pure brine
- Implementation in `packages/tendermint-light-client/solana/src/lib.rs::SolanaSignatureVerifier`

**Merkle Hashing Optimizations:**

- Skipping redundant validator set hash validation: **~290k CU saved**
  - `validator_sets_match` skip: ~145k CU
  - `next_validators_match` skip: ~145k CU
  - These hashes are pre-validated in `validate_basic()` and `check_trusted_next_validator_set()`

**Signature Pre-sorting:**

- Pre-sort signatures by validator address during serialization: **~60-80k CU saved**
- Avoids on-chain sorting during deserialization
- Implemented in `solana-ibc-types/src/borsh_header.rs::commit_to_borsh()`

**Validator Set Pre-sorting:**

- Pre-sorted validators from relayer: **~50k CU saved** per validator set
- Skips on-chain sorting by using pre-calculated total voting power

**Total Update Client Cost (Measured Real-World):**

The update client process is split into two phases:

**Phase 1: Prep Transactions (Parallel)**

- ALT (Address Lookup Table) creation + extension for address compression
- Header chunks upload (data transport for large headers)
- Ed25519Program signature pre-verification (optional, saves CU in assembly)

**Phase 2: Assembly Transaction**

- Deserializes and assembles header from uploaded chunks
- Verifies signatures (using pre-verification PDAs or brine-ed25519 fallback)
- Performs light client verification and updates state

**Noble (20 validators, 2/3 = ~14 signatures):**

- **Phase 1** - 19 parallel prep txs: 199,343 CUs, 0.000160 SOL (~3.3s)
- **Phase 2** - Assembly: 348,679 CUs, 0.0000064 SOL (~15s)
- **TOTAL: ~548k CUs, 0.000166 SOL** (~$0.025-0.033 USD at $150-200/SOL, ~18s)

**Celestia (100 validators, 2/3 = ~67 signatures):**

- **Phase 1** - 83 parallel prep txs: 893,310 CUs, 0.000715 SOL (~6s)
- **Phase 2** - Assembly: 1,270,083 CUs, 0.0000064 SOL (~16s)
- **TOTAL: ~2.16M CUs, 0.000721 SOL** (~$0.11-0.14 USD at $150-200/SOL, ~22s)

**Key Insights:**

- Cost scales roughly linearly with validator count (~5x validators = ~4x cost)
- Latency dominated by RPC throttling/rate limiting, not CU consumption
- Costs include base fees (5000 lamports/tx) + priority fees (variable, market-driven)
- Without optimizations: Large validator sets would exceed Solana's 1.4M CU transaction limit
- Optimizations enable: Support for validator sets of any size through parallelization

## Design Decisions

### Why On-Chain Signature Verification Instead of Ed25519Program?

This implementation uses `brine-ed25519` for on-chain Ed25519 signature verification (~30k CU per signature) instead of Solana's native Ed25519Program (FREE). This is a **fundamental architectural constraint**, not an optimization choice.

**Why Ed25519Program Doesn't Work for IBC:**

Solana's Ed25519Program is a precompile that verifies signatures included as Ed25519Program instructions in the current transaction. However, IBC light clients verify signatures from **external blockchain data** (Tendermint headers from Cosmos chains). These signatures:

- Come from Tendermint validators signing blocks on another chain
- Are embedded in header data uploaded via `upload_header_chunk`
- Cannot be reformulated as Ed25519Program instructions in the Solana transaction

**Why Transaction Chunking Doesn't Help:**

This implementation already uses multi-transaction chunking to upload large headers (see "Chunked Upload System" above). You might wonder: "If we're already coordinating multiple transactions for chunks, why not use Ed25519Program with multiple transactions?"

The key insight: **chunking is for DATA TRANSPORT, not signature verification**. The signatures are embedded INSIDE the serialized header data and can only be verified AFTER the header is fully assembled and deserialized in `assemble_and_update_client`.

Using Ed25519Program would require:

```
Current approach (brine-ed25519):
  1. Upload header chunks in PARALLEL (N transactions, ~2 block times)
  2. Assemble + verify all signatures (1 transaction, ~200k CU)
  Total latency: ~3 block times (~1.2 seconds)
Hypothetical Ed25519Program approach:
  1. Upload header chunks in PARALLEL (N transactions, ~2 block times)
  2. Assemble header, store in temp state (1 transaction)
  3. Extract signatures, create Ed25519Program verification txs (M transactions, SEQUENTIAL)
  4. Store verification results on-chain (additional rent costs)
  5. Final assembly to verify all passed (1 transaction)
  Total latency: ~(3 + M) block times (~1.2s + M*0.4s, where M ≈ 10-20)
```

This would create **double multi-transaction coordination** (chunks + signature verifications), with:

- **Significantly slower updates**: Current system uses parallel chunk upload (~2 blocks). Ed25519Program would add M sequential signature verification transactions, increasing latency by 4-8 seconds per update
- Additional state storage for verification results (rent costs likely exceed CU savings)
- More complex atomicity concerns (chunks AND signature verifications must all succeed)
- Risk of griefing (partial signature verifications succeed, final tx fails, relayer wasted resources)

The existing chunking system actually **strengthens** the case for brine-ed25519, as one layer of multi-tx complexity is manageable, but two would be exponentially harder and make updates much slower for relayers and users.

**Alternatives Considered:**

1. **Ed25519Program (native precompile)** - FREE compute units

   - ❌ Incompatible with external signature verification
   - Only works for signatures that are part of the transaction instruction set

2. **brine-ed25519 (on-chain library)** - ~30k CU per signature ✅ **CHOSEN**

   - ✅ Can verify any signature from external data
   - Uses optimized curve operations for efficiency
   - Typical update: ~200k CU total (10-20 signatures for 2/3 trust threshold)
   - Cost: ~$0.00001 USD per update
   - **Security**: Pulled from code-vm (MIT-licensed), audited by OtterSec, peer-reviewed by @stegaBOB and @deanmlittle

3. **Multi-transaction batching with Ed25519Program**
   - ❌ Impractical due to:
     - **Significantly slower**: Would add 4-8 seconds latency per update (10-20 sequential signature verification transactions)
     - Complex state management across transactions
     - Atomicity concerns (partial verification failures)
     - Coordination overhead
     - No cost benefit if verification state must be maintained on-chain

**Update Client Performance Across Different Tendermint Chains:**

The same Solana Tendermint light client implementation handles different chains with update client costs scaling based on validator count:

| Chain        | Validators | 2/3 Threshold | Prep Txs    | Total CUs | Total Cost (SOL) | USD Cost      | Latency |
| ------------ | ---------- | ------------- | ----------- | --------- | ---------------- | ------------- | ------- |
| **Noble**    | 20         | ~14 sigs      | 19 parallel | ~548k     | 0.000166         | ~$0.025-0.033 | ~18s    |
| **Celestia** | 100        | ~67 sigs      | 83 parallel | ~2.16M    | 0.000721         | ~$0.11-0.14   | ~22s    |

All costs measured on mainnet with real validator sets. Latency is highly dependent on RPC throttling/rate limiting.

For implementation details, see the `SolanaSignatureVerifier` in `packages/tendermint-light-client/update-client/src/solana.rs`.

## Integration Guide

### For Relayers

1. **Choose the target chain**: Determine which Tendermint chain you're relaying for (each chain_id has its own client)
2. Monitor Tendermint chain for new headers
3. Split header into 900-byte chunks
4. Upload all chunks in parallel for optimal performance
5. Optionally pre-verify signatures in parallel (recommended for 50+ validators)
6. Call `assemble_and_update_client` once all chunks are confirmed
7. Call `cleanup_incomplete_upload` to reclaim rent from chunks and signature verification accounts
8. Handle failures:
   - Retry failed chunks
   - Call `cleanup_incomplete_upload` to reclaim rent before retrying

**Multi-Chain Relaying**: Relayers can operate across multiple chains simultaneously. Each chain's uploads are isolated by chain_id in the PDA derivation.

**Performance Tip**: All chunks can be uploaded in parallel. A 9KB header (10 chunks of 900 bytes) can be uploaded in ~2 block times instead of ~10.

### For IBC Applications

1. **Specify the chain**: Reference the specific client via chain_id PDA
2. Call `verify_membership` for packet proofs from that chain
3. Check client is active (not frozen or expired) before relying on proofs
4. Monitor for client updates
5. **Multi-Chain Applications**: Can interact with multiple chains by referencing different chain_id PDAs
