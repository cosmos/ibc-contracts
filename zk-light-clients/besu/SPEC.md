# Besu QBFT ZK IBC Light Client — Implementation Specification v1.1.0

- Document ID: besu-qbft-zk-client-spec-v1.1.0-throughput
- Status: Normative implementation specification for throughput-optimized batched membership
- Audience: Rust engineers, Solidity engineers, ZK engineers, QA/security engineers, SREs, and coding agents
- Primary proving system: SP1
- Source consensus: Hyperledger Besu QBFT, block-header validator selection
- IBC baseline: cosmos/ibc-contracts Solidity Contracts solidity-v3.0.2, commit 39904319467b5fbdec8c34c43b66af6f1d256dba


## Document scope and implementation status

This file consolidates the v1.1 throughput specification, including its baseline QBFT and
non-membership requirements, into the single protocol reference. Architecture and operating
instructions are maintained in [docs/architecture.md](docs/architecture.md) and the linked guides.
Normative requirements below are acceptance targets, not a claim that every target has been
implemented or validated.

The reference IBC baseline named above is the public upstream release used by the v1.1 design.
The historical integration baseline used `partior-interop/ibc-contracts` at
`1d129afa1305069cb4ed709357922e5b44799dd8` through its contract submodule. Those revisions must not
be assumed interchangeable. Changing the build dependency requires checking the light-client ABI,
Router path/storage semantics, registration, and governance compatibility against the reference.
See [IBC dependency compatibility](docs/architecture.md#ibc-dependency-compatibility).

The layout in Section 5 describes the specification's intended component boundaries. Actual
source organization and current interface limitations are documented in architecture; the short
living documentation set is README, architecture, development, testing, deployment, integration,
and release. It does not require the former standalone security/validation/runbook documents.


Revision purpose: preserve the v1.0.1 QBFT/update/non-membership security model while changing membership proof granularity and off-chain orchestration for maximum throughput.

Primary changes in v1.1.0:

- variable-size batched membership, hard cap 32, preferred runtime batch size 10;
- one shared Router account proof per membership batch;
- canonical batchRoot plus per-item Merkle binding;
- one SP1 membership verification per batch with cached batchId for subsequent items;
- Proof API queue/coalescing/common-height/update-dedup rules;
- optional parallel update-client + membership proving after height freeze;
- no ICS26Router fork;
- updated benchmark/test matrix for batch sizes 1/5/10/20/32.



0. Normative language and implementation posture

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY use their RFC-style meanings.

MUST / MUST NOT
  Required for conformance.

SHOULD / SHOULD NOT
  May be changed only through a reviewed specification revision.

MAY
  Optional behavior that cannot weaken a MUST requirement.

0.1 Single-source rules

The implementation MUST use the following sections as the only normative definitions for their topics:

- Repository layout and code boundaries: Section 5
- Canonical Rust/public types: Section 6
- On-chain client/consensus state: Section 7
- QBFT update proof: Section 8
- Ethereum MPT verifier: Section 9
- IBC path/slot/commitments: Section 10
- Batched membership proof and batch commitment: Section 11
- Non-membership proof: Section 12
- SP1 Solidity verifier binding and membership envelopes: Section 13
- Solidity light-client behavior and verified-batch cache: Section 14
- Proof-generation tooling and Proof API batching/scheduling: Section 15
- Error/rejection behavior: Section 16
- Unit tests: Section 17
- SP1 execution/proving tests: Section 19
- Solidity integration tests: Section 20
- Besu-to-Besu E2E tests: Section 21
- Performance/resource gates and batch benchmarks: Section 22
- Reproducible build/VKeys: Section 23
- Deployment/release manifest: Section 24
- Coding-agent rules: Section 25
- Acceptance checklist: Section 26

If another comment, README, test-data file, source file, or older v1.0.1 implementation note conflicts with this specification, the implementation MUST stop with BLOCKED_ON_SPEC rather than silently choosing a different behavior.

0.2 Three-program rule

v1.1 still contains exactly three SP1 guest programs:

1. update-client
2. membership
3. non-membership

The update-client and non-membership proof statements remain logically unchanged from v1.0.1.

The membership program is changed from a single-item proof to a variable-size batched membership proof. One membership proof MUST authenticate between 1 and MAX_BATCH_MESSAGES entries under one proof height and one Router storage root.

There is no combined update-and-membership program in v1.1. There is no generic proofKind state program. Membership and non-membership remain separate ELFs and separate program verification keys.

0.3 No sequential-header replay

The update proof MUST NOT require or accept the sequence:

T+1, T+2, ..., H

The light-client algorithm verifies one target QBFT header H directly from a trusted validator authority at height T, using the trusted-set overlap rule and target-set QBFT quorum rule defined in Section 8.

The implementation MUST NOT add headers: Vec<QbftHeaderInput>, a vote-history replay engine, TransitionWitness, or trusted_transition_state_hash.

0.4 Supported QBFT validator mode

v1.1 supports only Besu QBFT block-header validator selection.

SUPPORTED:
  QBFT block-header validator selection.
  Validator addresses are carried in QBFT extraData.

NOT SUPPORTED:
  Contract-based validator selection.
  Runtime migration between header and contract validator modes.
  Replaying Besu's complete validator voting state from T to H.

A deployment using contract-based validator selection MUST be rejected before proof generation.

0.5 Throughput-optimization delta from v1.0.1

This specification intentionally changes the following v1.0.1 behaviors while preserving all other security statements unless explicitly overridden:

- Membership accepts a batch of 1..32 packet/acknowledgement membership entries instead of exactly one entry.
- One Router account proof is verified once per membership batch; each entry carries only its storage proof.
- Membership public values commit to a batch Merkle root and entry count instead of one pathDigest/expectedCommitment pair.
- BesuQBFTZKLightClient verifies the SP1 batch proof once, caches the authenticated batch identifier, and verifies a cheap per-item Merkle branch on each standard verifyMembership call.
- Upstream ICS26Router remains unchanged. Batching is implemented entirely behind the standard ILightClient boundary and in off-chain proof orchestration.
- A Proof API or equivalent scheduler MAY queue relayer proof intents, choose one common proof height, deduplicate update-client work, and build one membership batch.
- When target height H is frozen, update-client proving and membership proving MAY run concurrently. On-chain ordering remains updateClient(H) before recvPacket/ackPacket calls when H is not already stored.
- Non-membership remains single-item in v1.1. Batched timeout/non-membership is outside this revision.

0.6 Batch-size constants and preferred operating defaults

The membership guest MUST enforce:

MAX_BATCH_MESSAGES = 32
1 <= entries.len() <= MAX_BATCH_MESSAGES

Changing MAX_BATCH_MESSAGES changes the membership program and therefore requires a new membership ELF/VKey and release revision.

The Proof API preferred operating defaults are runtime configuration, not guest security constants:

preferredBatchSize = 10
maxBatchSize = 32
maxBatchWaitMs = 3000
allowedOperationalWaitRangeMs = 2000..5000
allowParallelUpdateAndMembership = true

preferredBatchSize is the initial tuning point for benchmarking, not a protocol limit. Operators SHOULD benchmark batch sizes 1, 5, 10, 20, and 32 before changing the production preferred size.

1. Scope and supported architecture

1.1 Goal

The repository implements a reusable SP1-backed IBC light client for two EVM chains that both use Hyperledger Besu QBFT and standard Ethereum account/storage Merkle-Patricia Tries.

+-----------------------------+                +-----------------------------+
| Besu QBFT Chain A           |                | Besu QBFT Chain B           |
|                             |                |                             |
| ICS26Router                 |                | ICS26Router                 |
| packet/ack/receipt state    |                | packet/ack/receipt state    |
+--------------+--------------+                +--------------+--------------+
               |                                              |
               |              IBC packet flow                 |
               +----------------------------------------------+
               |                                              |
               v                                              v
      Light client for B                             Light client for A
      - updateClient                                 - updateClient
      - verifyMembership                             - verifyMembership
      - verifyNonMembership                          - verifyNonMembership
      - SP1 proof verification                       - SP1 proof verification

The same source code MUST support either direction by deployment configuration. No program binary may contain a private chain address or private chain identifier.

1.2 Proof responsibilities

update-client
  trusted validator-set commitment at T
        +
  private full trusted validator set V_T
        +
  one target QBFT header H
        |
        v
  authenticate target stateRoot, timestamp, and validator-set commitment

membership
  authenticated stateRoot R_H
        +
  one Router account proof
        +
  1..32 exact IBC path/value entries
        +
  one storage proof per entry
        |
        v
  prove every packet/acknowledgement commitment exists under one Router storageRoot
        |
        v
  commit one canonical membership batchRoot

non-membership
  authenticated stateRoot R_H
        +
  exact receipt path
        +
  EIP-1186 account/storage witness
        |
        v
  prove exact receipt storage key is absent

1.3 In scope

- Besu QBFT block-header validator mode.
- Trusted-validator-set skip update T -> H.
- QBFT extraData decoding.
- Besu committed-seal signing digest reconstruction.
- secp256k1 signer recovery and validation.
- Target quorum and trusted-set overlap checks.
- Validator-set commitment hashing.
- Authenticated EVM stateRoot and block timestamp storage.
- Ethereum account MPT proofs.
- Batched Ethereum storage MPT membership with one shared Router account proof.
- Ethereum storage MPT non-membership proofs.
- Pinned IBC v2 packet/receipt/ack path derivation.
- Pinned ICS26Router storage-slot derivation.
- SP1 proof generation and EVM verification.
- Standard ILightClient integration through the pinned IBC contracts release.
- Proof-intent batching, common-height selection, update-job deduplication, and optional parallel update/membership proving.
- Standard ICS26Router multicall submission without forking ICS26Router.
- Bidirectional local Besu QBFT E2E conformance tests including batched packet and acknowledgement flows.

2. Pinned external dependencies and source basis

2.1 Required upstream baselines

The v1.1 repository MUST pin these baselines in the release manifest and dependency files.

Dependency

v1.1 baseline

Purpose

cosmos/ibc-contracts

solidity-v3.0.2, commit 39904319467b5fbdec8c34c43b66af6f1d256dba

ICS26Router, ILightClient, IBC path/commitment/storage semantics

SP1

v6.1.0

Guest execution, off-chain proving/setup, ELF/VKey generation

succinctlabs/sp1-contracts

v6.1.1

ISP1Verifier, verifier gateway/versioned verifier contracts

Hyperledger Besu source

Reference/conformance baseline: 26.5.0; another Besu release is allowed only when the release manifest pins it and the full Besu compatibility suite passes

QBFT extraData, committed-seal digest, signer/quorum behavior

Solidity compiler

0.8.28

Compiles the pinned IBC Solidity release and the SP1 verifier interface while providing one reproducible project compiler

The SP1 guest, SP1 SDK, and sp1-contracts verifier family MUST be version-compatible. v1 deliberately pins SP1 v6.1.0 with sp1-contracts v6.1.1: the latter is the patched contracts tag for the v6.1.0 verifier family. Newer SP1 SDK releases MUST NOT be substituted until a matching reviewed sp1-contracts verifier release is pinned. A version upgrade requires regenerated ELFs/VKeys and the full conformance suite.

2.2 Authoritative Besu behaviors implemented by v1.1

The implementation MUST reproduce these Besu QBFT behaviors:

QBFT extraData fields:
  vanity
  validators
  vote
  round
  committed seals

Committed-seal signing digest:
  canonical Ethereum header RLP
  with QBFT extraData re-encoded using the same
    vanity,
    validators,
    vote,
    round,
  but an EMPTY committed-seal list.

IMPORTANT:
  The committed-seal signing digest preserves the round number.
  The separate Besu on-chain BFT block hash rule that zeroes round is NOT
  the committed-seal signing digest and MUST NOT be used for signature checks.

Target QBFT quorum:
  ceil(2 * targetValidatorCount / 3)

Committed seals:
  every recovered committer must be in the target validator set;
  duplicate recovered committers are invalid.

The conformance reference is Besu 26.5.0. The implementation MUST use the 26.5.0 tagged source when creating golden QBFT test vectors and record the resolved full Git commit in release/manifest.json. A deployment may select another Besu release only after the release-specific test-data suite demonstrates identical behavior for every QBFT rule used by this client; otherwise it is BLOCKED\_ON\_SPEC.

2.4 Upstream IBC rule

The light client MUST compile against an unchanged, versioned upstream ILightClient interface and its required message types. An interface-only snapshot MAY be vendored with its upstream license and source revision. The full Router implementation MUST NOT be a standalone build or deployment dependency; it is supplied externally for E2E tests. For the pinned cosmos/ibc-contracts v1.1 baseline, the contract surface is normative and is exactly:

function updateClient(bytes calldata updateMsg)
    external
    returns (ILightClientMsgs.UpdateResult);

function verifyMembership(
    ILightClientMsgs.MsgVerifyMembership calldata msg_
) external returns (uint256);

function verifyNonMembership(
    ILightClientMsgs.MsgVerifyNonMembership calldata msg_
) external returns (uint256);

function misbehaviour(bytes calldata misbehaviourMsg) external;

function getClientState() external view returns (bytes memory);

The relevant pinned message types are:

struct MsgVerifyMembership {
    bytes proof;
    IICS02ClientMsgs.Height proofHeight;
    bytes[] path;
    bytes value;
}

struct MsgVerifyNonMembership {
    bytes proof;
    IICS02ClientMsgs.Height proofHeight;
    bytes[] path;
}

enum UpdateResult {
    Update,
    Misbehaviour,
    NoOp
}

verifyMembership and verifyNonMembership return the authenticated counterparty timestamp in Unix seconds. ICS26Router compares this value directly with the packet timeoutTimestamp, which is also in seconds. There is no nanosecond conversion in v1.1.
ICS02ClientUpgradeable stores each clientId -> ILightClient mapping separately. The v1 BesuQBFTZKLightClient instance therefore represents one registered counterparty client/route; its own ABI does not take a clientId parameter and its state MUST NOT be keyed by clientId internally.

3. Implementation architecture and code boundaries

This section exists to prevent security-critical checks from being placed in the wrong layer. The Proof API, relayer, RPC nodes, scheduler, and prover workers remain untrusted for correctness.

Maximum-throughput flow:

+-----------------------------+
| Source Besu JSON-RPC        |
| UNTRUSTED data source       |
+--------------+--------------+
               |
               | SendPacket / WriteAcknowledgement observations
               | eth_getBlockByNumber / eth_getProof
               v
+-----------------------------+
| Relayer / Proof API         |
| THROUGHPUT ORCHESTRATION    |
| - queue proof intents       |
| - deduplicate requests      |
| - coalesce 1..32 entries    |
| - freeze common height H    |
| - deduplicate update(H)     |
| - apply backpressure        |
+--------------+--------------+
               |
               | frozen batch + height H
               v
       +-------+-------+
       |               |
       v               v
+-------------+   +------------------+
| update-client|   | membership batch |
| SP1 proof   |   | SP1 proof        |
| T -> H      |   | N entries @ H    |
+------+------+   +---------+--------+
       |                    |
       | may prove in       |
       | parallel off-chain |
       +---------+----------+
                 |
                 v
+-------------------------------------+
| Relayer transaction builder         |
| ICS26Router.multicall                |
|  updateClient(H) if required         |
|  recvPacket/ackPacket item 1         |
|  recvPacket/ackPacket item 2         |
|  ...                                 |
+------------------+------------------+
                   |
                   v
+-------------------------------------+
| BesuQBFTZKLightClient               |
| SECURITY BINDING                    |
| - exact program VKey                |
| - stored ConsensusState[H]          |
| - verify SP1 batch once             |
| - cache verified batchId            |
| - verify per-item Merkle branch     |
+------------------+------------------+
                   |
                   v
+-------------------------------------+
| Upstream ICS26Router                |
| unchanged IBC packet lifecycle      |
+-------------------------------------+

3.1 Off-chain orchestration rule

The relayer/Proof API MAY decide when to seal a batch, which eligible height H to use, and which worker executes the proof. None of these decisions is trusted for correctness.

The scheduler MUST NOT be the only implementation of any security-critical check. The guest or Solidity binding MUST still enforce, as applicable:

validator-set hash binding
QBFT extraData validity
header RLP/signing digest
ECDSA validity/recovery
signer uniqueness
trusted overlap
QBFT target quorum
IBC path parsing
Router slot derivation
Router account MPT verification
storage MPT membership/non-membership
batch-root derivation
public-value derivation and on-chain context binding

3.2 Solidity rule

Solidity MUST NOT redo QBFT or MPT verification. It verifies the SP1 proof with the correct immutable VKey, binds the membership batch to a stored ConsensusState[H], and verifies only the cheap per-item membership branch required for the current Router-supplied path/value.

3.3 Common-height rule

A sealed membership batch MUST use exactly one proof height H and one state root R_H.

H MUST be frozen before witness collection begins. A newer source block appearing after H is frozen does not mutate the in-flight batch.

The batch builder MUST require every selected packet or acknowledgement commitment to be present at H. If any selected entry is absent, stale, or cannot be proven at H, that entry MUST be removed/split/requeued; the proving code MUST NOT fabricate a partial success.

3.4 Update/membership parallelism

When ConsensusState[H] does not yet exist on the destination, the Proof API MAY generate:

- update-client proof T -> H; and
- membership batch proof against source block H stateRoot

concurrently.

The membership proof does not need the update proof to finish off-chain. It only needs the frozen source block H and its stateRoot as witness context. Correctness is enforced on-chain because verifyMembership MUST compare the membership public stateRoot with ConsensusState[H].stateRoot after updateClient(H) executes earlier in the same multicall.

If ConsensusState[H] already exists, no update proof is required for that batch.

4. Supported protocol version and deployment configuration

There is no generic runtime IBCProfile abstraction in v1.1. The IBC path, commitment, Router storage layout, and ILightClient semantics are pinned to the upstream release in Section 2. Only values that genuinely vary per Besu-to-Besu deployment are supplied at runtime.

4.1 Runtime inputs for proof generation


A canonical TOML/YAML runtime configuration file is not required. The off-chain proving script MAY receive deployment-specific values through environment variables, CLI arguments, or an equivalent local mechanism.

The required logical runtime inputs are:

Source chain:
  source Besu RPC URL
  expected source chain ID
  source ICS26Router address
  validator mode = header-based QBFT

Destination chain:
  destination Besu RPC URL
  BesuQBFTZKLightClient address

Client policy:
  trustingPeriodSeconds
  maxClockDriftSeconds

SP1:
  proof mode = Groth16 EVM proof generated using the pinned SP1 v6.1.0 SDK
  verifier route = matching upstream Groth16 verifier through SP1VerifierGateway
  canonical ELF/VKey artifacts for the selected program

The RPC URLs, deployed addresses, bootstrap values, and any credentials are deployment/runtime values. They MUST NOT be compiled into the SP1 guest programs. Secrets MUST be supplied outside the repository and MUST NOT be committed.

v1.1 supports the pinned Besu-to-Besu IBC path shape in which MsgVerifyMembership.path and MsgVerifyNonMembership.path contain exactly one element: the raw packet, acknowledgement, or receipt commitment path. Deployments using a different Merkle-prefix/path shape are unsupported in v1.1 and require a specification revision.

4.2 Runtime validation

Before constructing a witness or generating a proof, the off-chain proving script MUST reject the request when:

source RPC is unavailable
destination RPC is unavailable when a destination read is required
eth_chainId != expected source chain ID
source Router address == zero
destination light-client address == zero
trustingPeriodSeconds or maxClockDriftSeconds do not fit uint64
selected validator mode is not header-based QBFT
selected proof mode is not the release-pinned SP1 proof mode

eth_chainId is an endpoint sanity check. It is NOT part of the QBFT committed-seal digest and MUST NOT be described as cryptographic chain-domain separation.


4.3 Throughput scheduler runtime configuration

The Proof API/scheduler MUST expose runtime configuration equivalent to:

preferredBatchSize = 10
maxBatchSize = 32
maxBatchWaitMs = 3000
allowParallelUpdateAndMembership = true
updateJobDeduplication = true
boundedPendingIntentQueue = true

Rules:

- maxBatchSize MUST NOT exceed the guest constant MAX_BATCH_MESSAGES.
- preferredBatchSize MAY be changed without changing the ELF as long as it remains <= maxBatchSize.
- maxBatchWaitMs SHOULD initially remain between 2000 and 5000 ms.
- worker concurrency is an operational parameter and MUST NOT be inferred from GPU count alone.
- a deployment MUST benchmark concurrent proof jobs before increasing active-job concurrency.

The scheduler MUST apply backpressure. It MUST NOT convert N incoming relayer events directly into N active SP1 proof jobs.

4.4 Repository-boundary rule for Proof API changes

The circuit/light-client repository MUST implement the batch-capable witnesses, proof artifacts, contract envelope, deterministic batch libraries, and a callable batch proving interface.

If the production Proof API service lives in another repository, Codex MUST NOT create a second unrelated production service inside this repository. It MUST instead:

1. implement the required batch-capable prover/tooling boundary here;
2. document the exact external Proof API changes required by Section 15; and
3. report them as EXTERNAL_CHANGE_REQUIRED in the completion report.

If the Proof API implementation already exists in this repository/workspace, Section 15 scheduling behavior is REQUIRED here.

5. Repository and module structure

The layout below defines the required logical separation between deterministic proof logic, SP1 programs, Solidity, test data, and off-chain proof-generation tooling. The proof-generation tool is a local CLI/script for development, testing, and proof production; it is not a REST/gRPC service.

besu-ibc-zk/
|
|-- README.md
|   Public project overview and minimal quick start.
|
|-- SPEC.md
|   This normative specification.
|
|-- LICENSE
|
|-- Cargo.toml
|   Root Rust workspace for shared crates, the three SP1 programs, and the local prover tool.
|
|
|-- Cargo.lock
|   Committed lockfile. Required for reproducible builds.
|
|-- rust-toolchain.toml
|   Exact Rust toolchain required by the pinned SP1 release.
|
|-- justfile
|   Canonical build/test/prove/e2e/release commands.
|
|-- .gitignore
|   Must ignore generated proofs, ELFs when not release-tracked, .e2e-state, local keys, logs.
|
|-- .github/
|   `-- workflows/
|       |-- rust.yml
|       |   cargo fmt, clippy, workspace unit/integration tests.
|       |-- solidity.yml
|       |   forge fmt/check/build/test and static analysis.
|       |-- sp1.yml
|       |   build/execute/prove/verify three SP1 programs on golden test data.
|       |
|       |-- e2e.yml
|       |   two local Besu QBFT chains and complete IBC proof flows.
|       `-- reproducibility.yml
|           clean build twice; compare ELF hashes and program VKeys.
|
|-- crates/
|   |
|   |-- types/
|   |   |-- Cargo.toml
|   |   |-- src/
|   |   |   |-- lib.rs
|   |   |   |-- update.rs
|   |   |   |-- membership.rs
|   |   |   |-- non_membership.rs
|   |   |   |-- qbft_header.rs
|   |   |   |-- mpt.rs
|   |   |   |-- constants.rs
|   |   |   `-- error.rs
|   |   `-- tests/
|   |
|   |       `-- roundtrip.rs
|   |
|   |-- qbft/
|   |   |-- Cargo.toml
|   |   |-- src/
|   |   |   |-- lib.rs
|   |   |   |-- extra_data.rs
|   |   |   |-- header.rs
|   |   |   |-- signing.rs
|   |   |   |-- signature.rs
|   |   |   |-- validator.rs
|   |   |   |-- quorum.rs
|   |   |   |-- trust.rs
|   |   |   `-- error.rs
|   |   `-- tests/
|   |       |-- extra_data.rs
|   |       |-- signing_digest.rs
|   |       |-- signatures.rs
|   |       |-- validators.rs
|   |       |-- quorum.rs
|   |       `-- trust.rs
|   |
|   |-- evm-mpt/
|   |   |-- Cargo.toml
|   |   |-- src/
|   |   |   |-- lib.rs
|   |   |   |-- rlp.rs
|   |   |   |-- nibble.rs
|   |   |   |-- hex_prefix.rs
|   |   |   |-- node.rs
|   |   |   |-- proof.rs
|   |   |   |-- inclusion.rs
|   |   |   |-- exclusion.rs
|   |   |   |-- account.rs
|   |   |   |-- storage.rs
|   |   |   `-- error.rs
|   |   `-- tests/
|   |       |-- rlp.rs
|   |       |-- hex_prefix.rs
|   |       |-- account_inclusion.rs
|   |       |-- storage_inclusion.rs
|   |       `-- storage_exclusion.rs
|   |
|   |-- ibc-storage/
|   |   |-- Cargo.toml
|   |   |-- src/
|   |   |   |-- lib.rs
|   |   |   |-- path.rs
|   |   |   |-- slot.rs
|   |   |   `-- error.rs
|   |   `-- tests/
|   |       |-- paths.rs
|   |       `-- storage_slots.rs
|   |
|   |-- batch-merkle/
|   |   |-- Cargo.toml
|   |   |-- src/
|   |   |   |-- lib.rs
|   |   |   |-- tree.rs
|   |   |   |-- proof.rs
|   |   |   `-- constants.rs
|   |   `-- tests/
|   |       `-- vectors.rs
|   |
|   `-- public-values/
|       |-- Cargo.toml
|       |-- src/
|       |   |-- lib.rs
|       |   `-- codec.rs
|       `-- tests/
|           |-- update.rs
|           |-- membership.rs
|           `-- non_membership.rs
|
|-- programs/
|   |-- update-client/
|   |   |-- Cargo.toml
|   |   `-- src/main.rs
|   |       SP1 guest entry point. Reads UpdateWitness, invokes qbft crate, commits UpdatePublicValues.
|   |
|   |-- membership/
|   |   |-- Cargo.toml
|   |   `-- src/main.rs
|   |       SP1 guest entry point. Derives the exact IBC slot, verifies account/storage inclusion,
|   |       and commits MembershipPublicValues.
|   |
|   `-- non-membership/
|       |-- Cargo.toml
|       `-- src/main.rs
|           SP1 guest entry point. Derives the exact receipt slot, verifies account plus exact MPT exclusion,
|           and commits NonMembershipPublicValues.
|
|-- tooling/
|   |-- prover-script/
|   |   |-- Cargo.toml
|   |   `-- src/
|           |-- main.rs
|           |-- rpc.rs
|           |-- witness.rs
|           |-- prover.rs
|           `-- artifact.rs
|           Local CLI/script used to query Besu RPC, build witnesses, execute/prove the three SP1 programs,
|           and output proof/public-value artifacts, including batched membership artifacts.
|   |
|   `-- proof-api-integration/
|       |-- Cargo.toml
|       `-- src/
|           |-- lib.rs
|           |-- intent.rs
|           |-- batch.rs
|           |-- scheduler.rs
|           `-- artifact.rs
|           Reusable batching/scheduling boundary for a production Proof API. If the production API lives
|           in another repository, this module is the contract that repository integrates against.
|
|-- contracts/
|   |-- foundry.toml
|   |-- remappings.txt
|   |-- src/
|   |   |-- BesuQBFTZKLightClient.sol
|   |   |-- BesuZKTypes.sol
|   |   |-- PublicValuesCodec.sol
|   |   `-- BatchMembershipLib.sol
|   |-- test/
|   |   |-- BesuQBFTZKLightClient.t.sol
|   |   |-- SP1VerifierIntegration.t.sol
|   |   `-- mocks/
|   |       `-- MockSP1Verifier.sol
|   |-- script/
|   |   `-- DeployLightClient.s.sol
|   `-- lib/
|       |-- ibc-contracts/
|       `-- sp1-contracts/
|
|-- test-data/
|   |-- qbft/
|   |   |-- valid/
|   |   `-- invalid/
|   |-- mpt/
|   |   |-- account/
|   |   |-- membership/
|   |   `-- non-membership/
|   |-- ibc/
|   |   |-- packet/
|   |   |-- acknowledgement/
|   |   `-- receipt/
|   `-- public-values/
|
|-- e2e/
|   |-- docker-compose.yml
|   |-- besu/
|   |   |-- chain-a-genesis.template.json
|   |   `-- chain-b-genesis.template.json
|   |-- contracts/
|   |   `-- TestIBCApp.sol
|   |-- scripts/
|   |   |-- generate-dev-networks.sh
|   |   `-- deploy-all.sh
|   `-- tests/
|       |-- update_client.rs
|       |-- packet_membership.rs
|       |-- batch_membership.rs
|       |-- acknowledgement.rs
|       |-- timeout.rs
|       |-- state_reuse.rs
|       |-- invalid_proofs.rs
|       `-- bidirectional.rs
|
|-- scripts/
|   |-- build-sp1.sh
|   |-- generate-vkeys.sh
|   |-- collect-test-data.sh
|   |-- release-workflow.sh
|   |-- verify-ibc-storage-layout.sh
|   |-- export-abis.sh
|   |-- check-reproducible-build.sh
|   `-- run-e2e.sh
|
|-- release/
|   `-- expected-vkeys.json
|
`-- docs/
    |-- architecture.md
    |-- DEVELOPMENT.md
    |-- TESTING.md
    |-- DEPLOYMENT.md
    |-- INTEGRATION.md
    `-- RELEASE.md

5.1 Dependency direction

programs/*
    -> types
    -> qbft OR evm-mpt/ibc-storage/batch-merkle/public-values

tooling/prover-script
    -> all shared crates
    -> RPC/network/SP1 SDK crates

tooling/proof-api-integration
    -> types/ibc-storage/batch-merkle/public-values
    -> MAY call prover-script libraries or the configured SP1 prover backend

contracts
    -> pinned ibc-contracts
    -> pinned sp1-contracts

shared deterministic crates
    -> MUST NOT depend on RPC/network crates

The guest entry points MUST be thin. Security logic belongs in deterministic shared crates so it can be unit-tested natively and exercised unchanged inside SP1. The proving script MAY perform equivalent checks as diagnostics, but those checks do not replace guest verification.

5.1.1 Root Cargo workspace

The root Cargo.toml MUST declare one Cargo workspace containing the deterministic Rust crates, all three SP1 programs, and the local prover tool.
[workspace]
resolver = "2"
members = [
  "crates/types",
  "crates/qbft",
  "crates/evm-mpt",
  "crates/ibc-storage",
  "crates/batch-merkle",
  "crates/public-values",
  "programs/update-client",
  "programs/membership",
  "programs/non-membership",
  "tooling/prover-script",
  "tooling/proof-api-integration",
]

5.1.2 SP1 guest build integration

scripts/build-sp1.sh MUST build exactly these three guest packages using the pinned SP1 build tooling:

programs/update-client
programs/membership
programs/non-membership

The resulting ELF artifacts MUST be the same artifacts consumed by the proving script, VKey generation, SP1 execution/proving tests, Solidity integration tests, and reproducibility checks. The repository MUST NOT maintain hand-copied ELF byte arrays or alternate guest sources.

The local proving script MUST load the canonical ELF corresponding to the selected operation:

update-client   -> update-client.elf
membership      -> membership.elf
non-membership  -> non-membership.elf

5.2 Generated artifacts

Generated release/test artifacts are written to an ignored artifacts/ directory:

artifacts/
  update-client.elf
  membership.elf
  non-membership.elf
  update-client.vkey
  membership.vkey
  non-membership.vkey
  proofs/
  public-values/
  abis/

release/expected-vkeys.json is committed and records expected ELF hashes and VKeys for a tagged release.

5.3 Local QBFT validator keys

E2E validator private keys MUST NOT be committed. generate-dev-networks.sh creates disposable keys under:

.e2e-state/

and .gitignore MUST exclude that directory.

6. Canonical types and serialization

6.1 General encoding rules

- Rust fixed hashes use raw 32-byte arrays.
- EVM addresses use raw 20-byte arrays in guest/shared crates.
- All explicit integer byte encodings in protocol hash functions are big-endian.
- Ethereum header/account/trie encoding uses canonical RLP, not ABI.
- Solidity mapping-slot derivation uses 32-byte ABI words exactly as specified.
- SP1 public values use Solidity ABI encoding of the structs in Section 6.6.
- Update and non-membership public values retain schemaVersion == 1.
- Batched membership public values use membershipSchemaVersion == 2.
- Off-chain/private witness transport uses the pinned SP1 serde I/O path.
- SP1 public values are ABI-encoded and committed as raw bytes with sp1_zkvm::io::commit_slice or the exact pinned equivalent.
- Decoders reject unsupported schema versions, length overflow, malformed encodings, and trailing bytes.

6.2 QBFT header input

The off-chain proving tooling obtains the target block with eth_getBlockByNumber(hex(H), false) and builds the exact typed header below. The guest reconstructs the canonical header RLP; the RPC-provided block hash is never authoritative.

pub struct QbftHeaderInput {
    pub parent_hash: [u8; 32],
    pub ommers_hash: [u8; 32],
    pub beneficiary: [u8; 20],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub logs_bloom: [u8; 256],
    pub difficulty: [u8; 32],
    pub number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: Vec<u8>,
    pub mix_hash: [u8; 32],
    pub nonce: [u8; 8],
    pub base_fee_per_gas: Option<[u8; 32]>,
    pub withdrawals_root: Option<[u8; 32]>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<[u8; 32]>,
    pub requests_hash: Option<[u8; 32]>,
    pub block_access_list_hash: Option<[u8; 32]>,
}

Rules from v1.0.1 remain unchanged: optional fork-tail fields are contiguous; numeric RLP is canonical/minimal; unknown additional tail fields are rejected until a specification revision adds them.

6.3 Update witness

pub struct UpdateWitness {
    pub schema_version: u32,
    pub trusted_height: u64,
    pub trusted_validator_set_hash: [u8; 32],
    pub trusted_validators: Vec<[u8; 20]>,
    pub target_header: QbftHeaderInput,
}

The update witness semantics remain unchanged from v1.0.1.

6.4 EIP-1186 proof witness types

pub struct Eip1186ProofWitness {
    pub account_proof: Vec<Vec<u8>>,
    pub storage_proof: Vec<Vec<u8>>,
}

pub struct MembershipEntryWitness {
    // Exact Router-supplied bytes[] path for one packet or acknowledgement.
    // v1.1 still requires path.len() == 1.
    pub path: Vec<Vec<u8>>,
    pub expected_commitment: [u8; 32],
    // Only the storage-trie proof for this entry. The Router account proof is shared.
    pub storage_proof: Vec<Vec<u8>>,
}

RPC-returned address, key, value, storageHash, codeHash, nonce, balance, or block hash fields MAY be retained as diagnostics but are never authoritative guest inputs.

6.5 State-proof witnesses

pub const MAX_BATCH_MESSAGES: usize = 32;

pub struct MembershipWitness {
    pub schema_version: u32,          // MUST equal 2
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],

    // Shared across every entry in this batch.
    pub account_proof: Vec<Vec<u8>>,

    // Canonically ordered 1..32 packet/ack entries.
    pub entries: Vec<MembershipEntryWitness>,
}

pub struct NonMembershipWitness {
    pub schema_version: u32,          // remains 1
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],
    pub path: Vec<Vec<u8>>,
    pub proof: Eip1186ProofWitness,
}

The membership guest MUST reject entries.len() == 0 or entries.len() > MAX_BATCH_MESSAGES.

The guest MUST derive every storage slot independently from each entry.path. The Proof API MUST NOT supply an authoritative storage slot.

6.6 Public values

The exact logical fields are ABI-encoded in this order and mirrored in BesuZKTypes.sol.

pub struct UpdatePublicValues {
    pub schema_version: u32,
    pub trusted_height: u64,
    pub trusted_validator_set_hash: [u8; 32],
    pub verified_height: u64,
    pub state_root: [u8; 32],
    pub block_timestamp: u64,
    pub new_validator_set_hash: [u8; 32],
}

pub struct MembershipPublicValues {
    pub schema_version: u32,          // MUST equal 2
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],
    pub batch_root: [u8; 32],
    pub entry_count: u32,
}

pub struct NonMembershipPublicValues {
    pub schema_version: u32,
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],
    pub path_digest: [u8; 32],
}

MembershipPublicValues deliberately does not expose every raw path/value pair. The batch_root commits to every validated pair according to Section 11.6, and each standard verifyMembership call proves its own pair belongs to that root.

6.7 Public-value ABI encoding

The SP1 guest MUST commit bytes exactly equivalent to Solidity:

abi.encode(BesuZKTypes.UpdatePublicValues(...))
abi.encode(BesuZKTypes.MembershipPublicValues(...))
abi.encode(BesuZKTypes.NonMembershipPublicValues(...))

Golden vectors MUST prove byte-for-byte equality between Rust and Foundry abi.encode.

The proving-tool/guest I/O contract remains:

// host/proving tool
let mut stdin = SP1Stdin::new();
stdin.write(&witness);

// guest
let witness: Witness = sp1_zkvm::io::read();

// guest public output
let encoded = PublicValues::abi_encode(&public_values);
sp1_zkvm::io::commit_slice(&encoded);

If SP1 v6.1.0 exposes equivalent methods through different imports, the implementation MUST preserve the byte contract above.

6.8 Membership batch leaf and node encoding

For each validated entry:

pathDigest = Keccak256(solidityAbiEncode(entry.path as bytes[]))

leaf = Keccak256(
    0x00
    || pathDigest
    || entry.expected_commitment
)

Internal nodes are positional and order-sensitive:

node = Keccak256(
    0x01
    || left
    || right
)

The canonical padding leaf is:

EMPTY_BATCH_LEAF = Keccak256(0x02)

For entry_count N, pad the ordered leaf list with EMPTY_BATCH_LEAF to the next power of two. For N == 1, the batch root is the single leaf and the per-item branch is empty.

The batch-merkle crate and Solidity BatchMembershipLib MUST implement exactly these bytes and no sorted-pair Merkle convention.

7. Light-client state and bootstrap

7.1 On-chain state model

The light client MUST NOT store full validator arrays.
Logical Solidity state:

struct ClientState {
    uint64 latestHeight;
    uint64 trustingPeriodSeconds;
    uint64 maxClockDriftSeconds;
}

struct ConsensusState {
    bytes32 stateRoot;
    uint64 blockTimestamp;       // source Besu header timestamp, seconds since Unix epoch
    bytes32 validatorSetHash;
}

Consensus states are indexed by IBC height. v1.1 maps:

revisionNumber = 0
revisionHeight = Besu block number

A nonzero revision number is unsupported in v1.1.
A consensus state exists only when it was installed at bootstrap or by a successful updateClient. The Solidity implementation MUST have an unambiguous existence test. The reference implementation uses blockTimestamp != 0 because bootstrap and guest validation both reject timestamp zero; it MUST NOT treat an all-zero mapping read as an existing state.

7.2 Route configuration held by the Solidity light client

Each deployed light-client instance MUST pin:

LOCAL_IBC_ROUTER          destination-side ICS26Router allowed to call the client
SOURCE_IBC_ROUTER         source-side ICS26Router proxy/account proved by state proofs
SP1_VERIFIER              upstream SP1VerifierGateway address configured
                          with the matching Groth16 verifier route
UPDATE_CLIENT_VKEY        update-client program VKey
MEMBERSHIP_VKEY           membership program VKey
NON_MEMBERSHIP_VKEY       non-membership program VKey

These values are immutable for a deployed client instance. Changing them requires a new deployment and standard IBC client registration/migration; there is no unrestricted setter.

7.3 Bootstrap state

Bootstrap is a governance/deployment trust action and is not proven by SP1.
Deployment MUST provide:

initialHeight T0 > 0
initialStateRoot R_T0 != 0
initialBlockTimestamp timestamp_T0 > 0
initialValidatorSetHash validatorSetHash_T0 != 0
trustingPeriodSeconds
maxClockDriftSeconds

Bootstrap tooling SHOULD fetch the selected trusted block from a known source Besu node, decode its header-mode validator set, compute the validator-set commitment with Section 8.3, and display all values for independent operator confirmation before deployment.

7.4 Consensus-state reuse

updateClient is not required for every packet operation.

If ConsensusState[H] exists:
  membership/non-membership may use H directly.

If ConsensusState[H] does not exist:
  relayer/prover must first submit a valid updateClient proof that stores H.

Several packet/acknowledgement/timeout proofs may reuse one accepted ConsensusState[H].

For maximum throughput, a membership cohort SHOULD deliberately choose one common H and prove all selected packet/acknowledgement entries under that one ConsensusState[H]. One accepted update may therefore serve many membership entries and many membership batches.

7.5 Trusting period

If trustingPeriodSeconds == 0, expiry is disabled.
Otherwise update acceptance MUST require:

ConsensusState[T].blockTimestamp + trustingPeriodSeconds > block.timestamp

An expired trusted state cannot be used to accept a new update.

7.6 Clock drift

Update acceptance MUST require:

newBlockTimestamp <= block.timestamp + maxClockDriftSeconds

The guest authenticates newBlockTimestamp as the target QBFT header timestamp; Solidity performs the local-time policy check.

7.7 Timestamp returned to IBC

The consensus state stores the source Besu header timestamp in Unix seconds. The pinned IBC light-client methods return this timestamp, and the pinned Router uses the returned verification timestamp for timeout handling.

verifyMembership(...)    -> uint256(ConsensusState[H].blockTimestamp)
verifyNonMembership(...) -> uint256(ConsensusState[H].blockTimestamp)

The returned timestamp MUST come from the same ConsensusState[H] whose stateRoot was bound to the membership/non-membership proof.


7.8 Common-height membership batches

For one membership batch:

proofHeight = H
stateRoot = R_H
sourceRouter = SOURCE_IBC_ROUTER

are shared by every entry.

The contract MUST reject any attempt to use one batch across different heights or roots. A valid batch generated against an unaccepted R_H is harmless until updateClient has stored exactly R_H at H.

8. updateClient SP1 program

8.1 Proof statement

The update-client program proves:

Given a validator-set commitment already trusted at height T, the supplied private validator array is exactly the trusted validator set V\_T; one newer Besu QBFT block header H has valid committed seals satisfying both the trusted-set overlap threshold and the exact Besu target-set quorum; therefore the header's stateRoot, timestamp, height, and target validator-set commitment may be accepted as a new consensus state.

The proof does not claim to replay every Besu validator vote or every intermediate header between T and H.

8.2 Required RPC and contract reads

Call

Called on

Required result

Trust status

eth_call to local light-client getter for ConsensusState[T]

Destination Besu

validatorSetHash_T, timestamp/state existence

Authoritative only because it is destination on-chain state; the returned value is still re-bound by Solidity

eth_getBlockByNumber(hex(T), false)

Source Besu

Trusted block T, especially QBFT extraData validator list

Untrusted witness source

eth_getBlockByNumber(hex(H), false)

Source Besu

Target header fields and QBFT extraData

Untrusted witness source

eth_chainId

Source Besu

Endpoint sanity check

Operational only, not signature-domain proof

The source RPC MUST retain historical headers back to every trusted height that the off-chain proving script may use. If the RPC cannot serve T, proof generation fails; the script MUST NOT substitute a current validator list. All block-number quantities emitted by the script MUST use minimal 0x-prefixed hexadecimal quantities.

8.3 Canonical validator-set commitment

The exact v1.1 commitment is:

validatorSetHash(V) = Keccak256(
    ASCII("BESU_QBFT_VALIDATOR_SET_V1")
    || U32_BE(len(V))
    || V[0]
    || V[1]
    || ...
    || V[n-1]
)

Each validator is exactly 20 raw address bytes, with no ABI padding and no textual hex representation.
Before hashing any validator set, the guest MUST require:

validatorCount >= 1
validatorCount MUST fit u32 because validatorSetHash encodes the count as U32_BE(len(V))
every address is exactly 20 bytes
addresses are strictly ascending by unsigned lexicographic 20-byte order
no duplicates
no zero address

The guest MUST NOT silently sort the witness. An unsorted witness is invalid.

8.4 Trusted-set overlap threshold

For trusted validator count n\_T:

trustedOverlapThreshold(n_T) = floor(n_T / 3) + 1

The guest counts unique recovered target-header committers that are members of V\_T and requires:

trustedSignerCount >= trustedOverlapThreshold(len(V_T))

This is a strict > 1/3 trusted overlap. No floating-point arithmetic is allowed.

8.5 Target QBFT quorum

For target validator count n\_H:

targetQuorum(n_H) = ceil(2 * n_H / 3)
                  = (2 * n_H + 2) / 3      // integer division

The implementation MAY use another overflow-safe integer expression only
if tests prove equivalence to ceil(2*N/3), including values from every
N mod 3 class and arithmetic edge cases. Floating-point arithmetic MUST
NOT be used.
The guest requires:

uniqueValidTargetCommitters >= targetQuorum(len(V_H))

Every recovered committer MUST be a member of V\_H. A seal from a non-target validator invalidates the target header; it is not merely ignored.

8.6 QBFT extraData parsing

For non-genesis target blocks, the guest decodes target\_header.extra\_data as the exact QBFT RLP list:

[
  vanityData,
  validators,
  vote,
  round,
  committedSeals
]

Required checks:

- extraData is canonical RLP.
- Root item is exactly one list.
- No trailing bytes exist after the root item.
- vanityData length equals the Besu QBFT vanity length used by the pinned Besu release.
- validators is a list of 20-byte addresses.
- vote is either the exact empty representation or a valid QBFT ADD/DROP vote encoding.
- round is a canonical non-negative RLP integer and fits u32.
- committedSeals is a list.
- - committedSeals is a list.
- each committed seal has the exact encoded length accepted by the selected Besu crypto format.

The guest uses the validators list as target validator set V\_H.
The guest does not replay the vote contained in H; the vote is nevertheless included unchanged in the committed-seal signing digest because it is part of the signed header.
8.7 Besu execution block-header canonicalization
Besu QBFT uses the Ethereum execution block-header format while replacing the consensus mechanism with QBFT. The guest MUST reconstruct the exact canonical RLP encoding of the Besu target block header from QbftHeaderInput. This reconstruction is required solely to derive the QBFT committed-seal signing digest; it does not implement or depend on Ethereum PoS consensus.
Canonical base order:

0  parentHash
1  ommersHash
2  beneficiary
3  stateRoot
4  transactionsRoot
5  receiptsRoot
6  logsBloom
7  difficulty
8  number
9  gasLimit
10 gasUsed
11 timestamp
12 extraData
13 mixHash
14 nonce
15 baseFeePerGas                 if present
16 withdrawalsRoot              if present
17 blobGasUsed                  if present
18 excessBlobGas                if present
19 parentBeaconBlockRoot        if present
20 requestsHash                 if present
21 blockAccessListHash          if present (Amsterdam / EIP-7928)

Optional tail fields MUST be contiguous. This program MUST reject a header with an unsupported field beyond the list above.
The target verified\_height, state\_root, and block\_timestamp public values are taken from this parsed/reconstructed header, not from independent off-chain metadata.

8.8 Committed-seal signing digest

The guest parses target extraData and constructs:

signingExtraData = QBFT.encode(
    vanity       = target.vanity,
    validators   = target.validators,
    vote         = target.vote,
    round        = target.round,
    committedSeals = []
)

It then replaces only the header's extraData field with signingExtraData, RLP-encodes the entire header canonically, and computes:

D_H = Keccak256(canonicalRlpHeaderWithEmptyCommitSeals)

The round is preserved. The on-chain BFT block hash routine that removes seals and zeroes round is a different Besu function and MUST NOT be used for committed-seal signature verification.

8.9 Commit-seal verification

For every seal in the target header:

Parse (r, s, recoveryId) from the exact Besu seal encoding.

Require 1 <= r < secp256k1\_n.

Require 1 <= s < secp256k1\_n.

Require recoveryId == 0 or recoveryId == 1.

Reject every other recovery identifier, including 27 and 28.

The implementation MUST NOT normalize 27/28 to 0/1. The accepted recovery
identifier encoding MUST exactly match the pinned Besu 26.5.0 QBFT
commit-seal decoder.

Recover the candidate secp256k1 public key from D\_H and the signature.

Verify the ECDSA equation for the recovered key and D\_H; recovery alone is not treated as a separate trust assertion.

Derive the Ethereum address as the last 20 bytes of Keccak256(uncompressedPubkeyX || uncompressedPubkeyY) with the 0x04 prefix omitted.

Reject address zero.

Reject if the signer is not a member of V\_H.

Reject if the recovered signer appeared for an earlier seal.

Count the signer once toward target quorum.

If the signer is also a member of V\_T, count it once toward trusted overlap.

v1.1 does not add an independent low-S rejection beyond what is necessary to reproduce a mathematically valid Besu seal. Duplicate signer identity, not duplicate signature bytes, is the quorum-safety rule. Tests MUST include malleated/high-S cases and prove they cannot create extra quorum weight.

8.10 Height and time checks inside the guest

The guest MUST require:

schema_version == SCHEMA_VERSION
trusted_height > 0
target_header.number > trusted_height
target_header.timestamp > 0

The guest does not compare to destination wall-clock time because wall-clock policy is enforced by Solidity in the transaction that verifies the proof.

8.11 Trusted validator binding inside the guest

Before using trusted\_validators for overlap:

computedTrustedHash = validatorSetHash(trusted_validators)
require computedTrustedHash == trusted_validator_set_hash

This check prevents a prover from choosing an arbitrary trusted validator array while claiming the on-chain commitment of another set.

8.12 Target validator commitment

After parsing and validating target V\_H:

newValidatorSetHash = validatorSetHash(V_H)

The full V\_H array is not public output and is not stored on-chain.

8.13 Guest algorithm

The normative guest algorithm is:

1. Read UpdateWitness and require schema_version == 1.
3. Validate trusted_height > 0.
4. Validate trusted_validators as a canonical validator list.
5. Compute validatorSetHash(trusted_validators).
6. Require it equals trusted_validator_set_hash.
7. Validate target_header.number > trusted_height.
8. Validate target_header optional-field layout and field lengths for the reference Besu 26.5.0 header schema.
9. Parse target_header.extra_data as canonical QBFT extraData.
10. Extract and validate target validator set V_H.
11. Extract vote, round, and committed seals.
12. Re-encode QBFT extraData with identical vanity/validators/vote/round and empty seals.
13. Canonically RLP-encode the complete target header using that signing extraData.
14. Compute D_H = Keccak256(signing header RLP).
15. For every target commit seal, recover and validate the signer.
16. Reject any signer outside V_H.
17. Reject duplicate recovered signer addresses.
18. Count unique V_H signers.
19. Count the subset of those signers that are in trusted V_T.
20. Require V_H signer count >= ceil(2 * |V_H| / 3).
21. Require trusted signer count >= floor(|V_T| / 3) + 1.
22. Compute newValidatorSetHash = validatorSetHash(V_H).
23. Construct UpdatePublicValues from trusted height/hash and target header number/stateRoot/timestamp/new hash.
24. ABI-encode UpdatePublicValues exactly as Section 6.7.
25. Commit those bytes as SP1 public values.

8.14 Solidity updateClient acceptance

Given a canonical SP1ProofEnvelope, updateClient(bytes) MUST execute this acceptance flow:

1. Require caller == LOCAL_IBC_ROUTER.
2. Strictly decode exactly one canonical SP1ProofEnvelope.
3. Strictly decode UpdatePublicValues and require schemaVersion == 1.
4. Load ConsensusState[public.trustedHeight]; require it exists.
5. Require public.trustedValidatorSetHash == stored.validatorSetHash.
6. If trustingPeriodSeconds > 0, require
      stored.blockTimestamp + trustingPeriodSeconds > block.timestamp.
7. Require public.verifiedHeight > public.trustedHeight.
8. Require public.blockTimestamp > stored.blockTimestamp.
9. Require public.blockTimestamp <= block.timestamp + maxClockDriftSeconds.
10. Call
      ISP1Verifier(SP1_VERIFIER).verifyProof(
          UPDATE_CLIENT_VKEY,
          envelope.publicValues,
          envelope.proofBytes
      ).
11. After proof verification, inspect ConsensusState[public.verifiedHeight].
12. If a state already exists at verifiedHeight:
      a. if stateRoot, blockTimestamp, and validatorSetHash exactly equal
         public.stateRoot, public.blockTimestamp, and public.newValidatorSetHash,
         return ILightClientMsgs.UpdateResult.NoOp without rewriting state;
      b. otherwise revert ConflictingConsensusState(verifiedHeight).
         v1.1 does not freeze the client or claim to prove misbehaviour.
13. If no state exists at verifiedHeight, store:
      ConsensusState[verifiedHeight] = {
          stateRoot: public.stateRoot,
          blockTimestamp: public.blockTimestamp,
          validatorSetHash: public.newValidatorSetHash
      }.
14. If verifiedHeight > latestHeight, set latestHeight = verifiedHeight.
15. Emit ClientUpdated(...).
16. Return ILightClientMsgs.UpdateResult.Update.

There are no processedTimes / processedHeights mappings in v1.1 because the pinned IBC interface used here has no delay-period verification semantics requiring them.
v1.1 does not implement conflicting-header misbehaviour evidence. A conflicting already-stored height fails closed and is never overwritten.

8.15 What the update proof does not prove

- It does not prove ancestry from T to H with parent-hash links.
- It does not replay validator votes between T and H.
- It does not prove EVM chainId from the header.
- It does not prove IBC Router storage.
- It does not use an RPC-reported block hash as a trust anchor.

Its security derives from the recent trusted validator authority plus target QBFT finality under the assumptions in Sections 4 and 8.

9. Common Ethereum MPT implementation

The evm-mpt crate is shared by membership and non-membership and MUST have no RPC dependency.

9.1 Root chain

authenticated Besu stateRoot R_H
        |
        | secure account key = Keccak256(sourceRouterAddress)
        v
Router account MPT proof
        |
        v
RLP account [nonce, balance, storageRoot, codeHash]
        |
        | extract storageRoot S_H
        v
secure storage key = Keccak256(logicalStorageSlot K)
        |
        v
Router storage MPT proof
        |
        +--> membership: exact commitment value
        |
        `--> non-membership: exact key absent

9.2 Secure trie keys

Account trie:

accountKeyBytes = Keccak256(raw20ByteRouterAddress)
accountPath = nibbles(accountKeyBytes)

Storage trie:

storageKeyBytes = Keccak256(ABIEncode32(logicalStorageSlot))
storagePath = nibbles(storageKeyBytes)

The logical slot K itself is passed to eth\_getProof. The node performs the secure-trie hashing internally. The guest repeats that secure-key hashing before MPT traversal.

9.3 Canonical RLP requirements

The MPT verifier MUST reject non-canonical RLP. At minimum:

- Single byte 0x00..0x7f must not be encoded as a one-byte string wrapper.
- Short/long string length forms must be minimal.
- Short/long list length forms must be minimal.
- Length-of-length must contain no leading zero.
- Declared lengths must match available bytes exactly.
- Integer consumers reject leading-zero integer encodings.
- Root node decode must consume the complete node byte slice.
- Recursive length arithmetic must use checked arithmetic.

9.4 MPT node model

The verifier supports exactly:

Branch node:
  RLP list length 17
  children[0..15]
  value[16]

Extension or leaf node:
  RLP list length 2
  compactEncodedPath
  childReference OR leafValue

Any other list length is invalid.

9.5 Hex-prefix compact path

The decoder MUST implement Ethereum hex-prefix encoding:

flag high nibble:
  0 = extension, even path
  1 = extension, odd path
  2 = leaf, even path
  3 = leaf, odd path

For even paths the low nibble of the first byte MUST be zero padding. Invalid flags/padding are rejected.

9.6 Child references

For a branch/extension child reference:

empty bytes:
  no child

length < 32:
  inline RLP child; child bytes themselves are the reference

length == 32:
  Keccak256(childRlp) must equal the 32-byte reference

length > 32:
  invalid child reference

The root itself is addressed by its 32-byte hash.

9.7 Proof traversal and supplied node usage

The verifier MAY build a map from Keccak256(nodeRlp) to supplied node
bytes for hashed references. It MUST:

- use checked arithmetic for RLP length and traversal calculations;
- never allocate based on an unchecked embedded RLP length declaration;
- reject duplicate supplied node hashes carrying different bytes;
- validate inline children directly from the parent reference;
- reject a missing referenced hashed node;
- reject malformed nodes even if their hash is referenced;
- reject arithmetic/path overflows;
- return an explicit Present(value) or Absent result;
- never return Absent merely because decoding failed.

Extraneous unused supplied nodes SHOULD be rejected so the test-data format remains canonical and witness ambiguity is minimized.

9.8 Account proof

verify\_account(root, address, accountProof) MUST return an authenticated account object only when the account exists.
Account leaf value decodes as exactly four RLP items:

[ nonce, balance, storageRoot, codeHash ]

Required checks:

nonce and balance are canonical unsigned RLP integers
storageRoot exactly 32 bytes
codeHash exactly 32 bytes
account result must be Present

Membership and non-membership proofs MUST reject an absent Router account. Proving that the entire Router account is absent is not a valid IBC receipt non-membership proof.

9.9 Storage inclusion result

A present storage leaf value is an RLP-encoded unsigned integer. The verifier MUST:

Decode it canonically as unsigned U256.

Convert to a 32-byte big-endian, left-zero-padded storage word.

Return Present([u8;32]).

For membership, the word MUST equal expected\_commitment exactly.

9.10 Storage exclusion result

verify\_storage\_non\_membership succeeds only for an authenticated MPT exclusion of the exact secure storage key. Valid exclusion forms include:

- a required branch child is empty;
- an extension compact path diverges from the requested remaining key;
- a leaf compact path diverges from the requested remaining key;
- the storage trie is the canonical empty trie root and the account's storageRoot commits to it.

A parser error, hash mismatch, missing proof node, or proof for another key is not exclusion.

9.11 Zero versus absence

The API must preserve the distinction:

pub enum ProofResult {
    Present([u8; 32]),
    Absent,
}

For non-membership, only Absent is accepted.
Present([0u8;32]) MUST NOT be silently converted to Absent, even though normal Ethereum state omits zero-valued storage slots. This fail-closed rule keeps the circuit statement explicit.

10. IBC path, commitment, and Router-slot derivation

This section is pinned to the upstream IBC release in Section 2. Codex MUST implement the path and storage-slot rules once in crates/ibc-storage; the proving script, native tests, and both state-proof guests use the same deterministic implementation.

10.1 v1.1 path shape

For the supported Besu-to-Besu pairing, ILightClientMsgs.MsgVerifyMembership.path and MsgVerifyNonMembership.path MUST contain exactly one element:

path.length == 1
terminalPath = path[0]

That element is the raw commitment path used as the key input to the pinned ICS26Router commitment store. A deployment whose Router/counterparty configuration produces a different bytes[] path shape is unsupported in v1.
The off-chain proving script MUST pass the exact bytes[] path from the IBC operation into the witness. The guest MUST NOT accept a script-supplied terminal path or storage slot independently of that array.

10.2 Terminal paths

The exact terminal-path rules are:

Packet:
  packetPath = sourceClientId || 0x01 || U64_BE(sequence)

Receipt:
  receiptPath = destClientId || 0x02 || U64_BE(sequence)

Acknowledgement:
  acknowledgementPath = destClientId || 0x03 || U64_BE(sequence)

U64\_BE is exactly eight bytes, most significant byte first.

10.3 Path public binding

The proof binds the exact Router-supplied bytes[] value:

pathDigest = Keccak256(solidityAbiEncode(path as bytes[]))

Solidity recomputes keccak256(abi.encode(msg\_.path)) and requires equality with the SP1 public value.

10.4 Terminal-path structural validation

The guest MUST first require:

path.len() == IBC_PATH_ELEMENTS == 1
path[0] is non-empty

It then parses terminalPath = path[0].
For membership, the type byte MUST be packet 0x01 or acknowledgement 0x03.
For non-membership, the type byte MUST be receipt 0x02.
For every terminal path:

- total length >= 10 bytes (at least one client-id byte + one type byte + eight sequence bytes);
- last 8 bytes are sequence U64_BE;
- byte immediately before the sequence is the type byte;
- preceding bytes form a non-empty client ID;
- client ID bytes satisfy the identifier grammar of the pinned IBC release;
- there are no bytes after the eight-byte sequence.

crates/ibc-storage/tests/paths.rs MUST contain golden vectors from the pinned upstream Solidity implementation. The Rust parser MUST NOT invent a broader grammar.

10.5 Logical Router storage slot

hashedPath = Keccak256(terminalPath)

K = Keccak256(
      ABIEncode32(hashedPath)
      || ABIEncode32(IBCSTORE_STORAGE_SLOT)
    )

where:

IBCSTORE_STORAGE_SLOT =
0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600

K is the logical Solidity storage slot supplied to:

eth_getProof(sourceRouter, [K], hex(H))

The off-chain proving script derives K because the RPC requires the key. K is NOT a guest trust input. The guest independently re-derives K from path[0] and the pinned storage layout.

10.6 Storage layout verification gate

scripts/verify-ibc-storage-layout.sh MUST:

Read an explicitly supplied external ibc-contracts checkout and verify its selected exact revision. Never fetch it as a light-client submodule. This is an integration/release test gate, separate from standalone deployment preflight.

Locate the IBCStoreStorage.commitments mapping used by ICS26Router.

Confirm the ERC-7201 namespace/base slot equals IBCSTORE\_STORAGE\_SLOT above.

Confirm the commitment mapping key/value types are bytes32 -> bytes32.

Confirm packet/receipt/acknowledgement terminal path type bytes and eight-byte big-endian sequence encoding.

Confirm the supported Besu-to-Besu IBC call path supplies one raw path element to the light client.

Confirm a Foundry reference slot calculation matches Rust/SP1 golden vectors.

Exit nonzero on any difference.

A release cannot proceed when this gate fails.

10.7 Expected packet and acknowledgement commitment binding

The Besu ZK light-client implementation does not independently reconstruct packet or acknowledgement commitments from the packet fields. The pinned upstream IBC contracts are authoritative for deriving those commitments.

For membership verification, the IBC operation supplies the exact expected commitment in MsgVerifyMembership.value. The supported call path requires this value to be exactly 32 bytes.

The membership proof therefore establishes the narrower and security-relevant statement:

expectedCommitment = exact 32-byte value supplied by the IBC operation

under authenticated stateRoot R_H:
  source Router account exists
  -> authenticated Router storageRoot
  -> path-derived logical slot K exists
  -> authenticated stored word at K == expectedCommitment

For each batched membership entry, the proving tooling MUST copy the exact expected 32-byte commitment into MembershipEntryWitness.expected_commitment. The membership guest MUST prove equality with the authenticated Router storage word and commit the validated (pathDigest, expectedCommitment) pair into batchRoot. Solidity MUST bind the exact 32 bytes supplied by msg_.value by recomputing that entry's batch leaf and verifying its Merkle branch against the authenticated batchRoot.

The Besu ZK implementation MUST NOT apply an additional packet hash or acknowledgement hash to those 32 bytes. Packet/acknowledgement commitment algorithms remain owned by the pinned upstream IBC implementation.

Tests MUST include packet and acknowledgement examples whose expected 32-byte commitment values are obtained from the pinned upstream IBC implementation, so that the integration proves the correct value is passed through unchanged.

10.8 Receipt non-membership

Timeout verification needs the exact receipt path and cryptographic proof of absence. The non-membership program does not accept or predict an expected receipt value.

11. Membership SP1 program - batched membership

11.1 Supported statements

The membership program supports exactly:

- packet commitment membership; and
- acknowledgement commitment membership.

One execution proves 1..32 such entries under one Besu proof height H and one Router account/storage root.

It does not prove arbitrary Router storage and does not prove receipt non-membership.

11.2 Batch invariants

Every batch MUST satisfy:

1 <= entry_count <= MAX_BATCH_MESSAGES == 32
all entries use the same proof_height H
all entries use the same state_root R_H
all entries use the same source_router
all paths have exactly one element
path type is packet 0x01 or acknowledgement 0x03
expected commitment is exactly 32 bytes
no duplicate pathDigest exists in one batch
entries are strictly ascending by pathDigest; ties are invalid

The guest MUST NOT silently sort entries. The Proof API is responsible for constructing canonical order before proving; the guest enforces it.

11.3 Required reads and RPCs

The preferred RPC acquisition is one EIP-1186 call for all batch slots:

eth_getProof(sourceRouter, [K1, K2, ..., KN], hex(H))

The response provides:

- one Router account proof; and
- N storageProof entries, one for each requested logical slot.

The tooling MUST require the returned storageProof keys to correspond exactly to the requested K values. A node/RPC that reorders storageProof entries MAY be supported only if the tooling explicitly matches by returned key and then restores canonical entry order; it MUST NOT assume positional equality silently.

The batch membership proof MAY be generated in either of these states:

A. ConsensusState[H] already exists on destination:
   read stored R_H and require it equals source block H stateRoot.

B. ConsensusState[H] does not yet exist:
   fetch source block H, use its stateRoot as witness input, and generate membership concurrently with update-client(H). On-chain verification later binds the proof to the stored state.

11.4 Off-chain witness construction

1. Receive a sealed cohort of 1..32 membership intents.
2. Freeze one proof height H for the entire cohort.
3. Fetch source block H and extract candidate stateRoot R_H.
4. If ConsensusState[H] already exists, require stored.stateRoot == R_H.
5. Load configured SOURCE_IBC_ROUTER.
6. For each intent:
   a. validate path.length == 1;
   b. validate packet/ack terminal path according to Section 10;
   c. require expected commitment is exactly 32 bytes;
   d. compute pathDigest;
   e. derive hashedPath and logical slot K.
7. Deduplicate by pathDigest. Duplicate entries MUST NOT count toward batch size.
8. Sort entries strictly by pathDigest.
9. Call eth_getProof(sourceRouter, [K1..KN], hex(H)).
10. Require a non-empty accountProof and exactly N matching storageProof entries.
11. Normalize one shared accountProof and one storage proof per canonical entry.
12. Build MembershipWitness(schema=2, H, R_H, sourceRouter, accountProof, entries).
13. Optionally SP1-execute locally.
14. Generate one membership SP1 proof.
15. From the canonical entries, generate one positional Merkle branch per entry for relayer submission.

The tooling MAY deduplicate identical MPT node blobs across storage proofs internally for memory/serialization optimization, but the guest-visible semantics MUST remain identical.

11.5 Guest algorithm

1. Read MembershipWitness and require schema_version == 2.
2. Require proof_height > 0 and source_router != zero.
3. Require 1 <= entries.len() <= 32.
4. Verify source_router account inclusion under state_root exactly once using account_proof.
5. Strictly decode the authenticated account and extract Router storageRoot S_H.
6. Initialize an ordered leaf list.
7. For each entry in order:
   a. validate path exactly according to Section 10;
   b. require path type packet 0x01 or acknowledgement 0x03;
   c. compute pathDigest = keccak256(abi.encode(path));
   d. require pathDigest is strictly greater than the previous pathDigest;
   e. compute hashedPath and logical slot K;
   f. compute secure storage-trie key keccak256(ABIEncode32(K));
   g. verify exact storage inclusion under the shared S_H using entry.storage_proof;
   h. require Present(actualStoredWord), not Absent;
   i. require actualStoredWord == entry.expected_commitment;
   j. compute the Section 6.8 batch leaf and append it.
8. Pad leaves to the next power of two using EMPTY_BATCH_LEAF.
9. Compute the positional batch_root using Section 6.8.
10. Construct MembershipPublicValues(schema=2, proofHeight, stateRoot, sourceRouter, batchRoot, entryCount).
11. ABI-encode and commit the public values.

11.6 Canonical batch commitment

The batch root is a commitment to the exact ordered set of validated (pathDigest, expectedCommitment) pairs.

Security properties:

- changing any path changes its leaf;
- changing any expected commitment changes its leaf;
- changing entry order changes the positional root;
- dropping or inserting an entry changes entryCount and normally the root;
- duplicate pathDigest entries are rejected before root construction.

The SP1 proof proves that every leaf under batchRoot corresponds to an actual authenticated Router storage value under the same R_H.

11.7 Membership public values

MembershipPublicValues contains exactly:

schemaVersion = 2
proofHeight H
stateRoot R_H
sourceRouter
batchRoot
entryCount

No raw MPT nodes and no raw path arrays are public values.

11.8 Membership proof envelope used by ILightClient

Because upstream ICS26Router calls verifyMembership once per packet/acknowledgement, msg_.proof MUST carry enough information to bind the current item to the already-proved batch.

Logical Solidity type:

struct MembershipBatchProofEnvelope {
    uint8 mode;                // 1 = FULL, 2 = CACHED
    bytes publicValues;        // FULL only; empty for CACHED
    bytes proofBytes;          // FULL only; empty for CACHED
    bytes32 batchRoot;
    uint32 entryCount;
    uint32 leafIndex;
    bytes32[] merkleProof;
}

Rules:

FULL mode:
  publicValues and proofBytes MUST be non-empty.
  Solidity verifies the membership SP1 proof and caches the computed batchId.

CACHED mode:
  publicValues and proofBytes MUST be empty.
  Solidity requires the computed batchId to already be cached as verified.

Both modes MUST verify the current Router-supplied (msg_.path, msg_.value) leaf against batchRoot using leafIndex and merkleProof.

11.9 Batch identifier

The contract computes:

batchId = keccak256(
    abi.encode(
        uint32(2),
        uint64(H),
        bytes32(ConsensusState[H].stateRoot),
        address(SOURCE_IBC_ROUTER),
        bytes32(batchRoot),
        uint32(entryCount),
        bytes32(MEMBERSHIP_VKEY)
    )
)

The verified-batch cache is local to one light-client instance. A batch verified by another light-client contract does not populate this cache.

11.10 Solidity verifyMembership binding

For every call, the light client MUST:

1. require caller == LOCAL_IBC_ROUTER;
2. require msg_.proofHeight.revisionNumber == 0;
3. let H = msg_.proofHeight.revisionHeight and require ConsensusState[H] exists;
4. require msg_.path.length == 1;
5. require msg_.value.length == 32;
6. strictly decode MembershipBatchProofEnvelope;
7. require 1 <= entryCount <= 32;
8. require leafIndex < entryCount;
9. recompute batchId from H, stored.stateRoot, SOURCE_IBC_ROUTER, batchRoot, entryCount, and MEMBERSHIP_VKEY;
10. if FULL:
    a. strictly decode MembershipPublicValues and require schemaVersion == 2;
    b. require public.proofHeight == H;
    c. require public.stateRoot == stored.stateRoot;
    d. require public.sourceRouter == SOURCE_IBC_ROUTER;
    e. require public.batchRoot == envelope.batchRoot;
    f. require public.entryCount == envelope.entryCount;
    g. verify the SP1 proof with MEMBERSHIP_VKEY;
    h. set verifiedMembershipBatches[batchId] = true;
    i. emit MembershipBatchVerified(batchId, H, batchRoot, entryCount);
11. if CACHED:
    a. require publicValues.length == 0 and proofBytes.length == 0;
    b. require verifiedMembershipBatches[batchId] == true;
12. compute pathDigest = keccak256(abi.encode(msg_.path));
13. interpret msg_.value as the exact 32-byte expected commitment without re-hashing;
14. compute the leaf using Section 6.8;
15. verify the positional Merkle branch at leafIndex against batchRoot;
16. return uint256(stored.blockTimestamp).

A proof produced by update-client or non-membership MUST fail because FULL mode selects MEMBERSHIP_VKEY.

11.11 One full envelope per submitted batch

For maximum on-chain throughput, the relayer SHOULD submit one FULL membership envelope before CACHED envelopes that reference the same batchId.

A typical transaction is:

ICS26Router.multicall([
    updateClient(H),                    // omit if H already stored
    recvPacket(P1, FULL batch proof),   // verifies SP1 once and caches batchId
    recvPacket(P2, CACHED branch),
    recvPacket(P3, CACHED branch),
    ...
])

The same pattern applies to acknowledgement membership through ackPacket.

11.12 Failure atomicity

If all calls are submitted in one multicall and a later packet call reverts, normal EVM transaction atomicity reverts the earlier batch-cache write as well. The implementation MUST NOT rely on an event emitted by a reverted transaction as proof of cache existence.

12. Non-membership SP1 program

12.1 Supported statement

The non-membership program proves one statement only:

The exact destination receipt commitment storage key derived from the Router-supplied receipt path is absent under the authenticated source Router storage root at proof height H.

A raw RPC value of zero is not proof of absence.

12.2 Required reads and RPCs

Call

Called on

Purpose

eth_call local light-client consensus-state getter at H

Chain executing timeout verification

Read accepted stateRoot R_H and timestamp

eth_getProof(sourceRouter, [K], hex(H))

Counterparty/source-of-state Besu

Obtain Router account proof and receipt-slot exclusion witness

optional eth_getStorageAt(sourceRouter, K, hex(H))

Counterparty/source-of-state Besu

Diagnostic precheck only; never proof evidence

12.3 Off-chain proving-script witness construction

1. Receive proofHeight H and the exact receipt bytes[] path from the timeout operation.
2. Require path.length == 1 and validate the exact Section 10 receipt-path structure.
3. Read local ConsensusState[H]; require it exists and obtain R_H.
4. Load configured SOURCE_IBC_ROUTER.
5. Parse terminalPath = path[0] and require receipt type byte 0x02.
6. Derive logical slot K.
7. Call eth_getProof(sourceRouter, [K], hex(H)).
8. Require exactly one storageProof entry and returned key == K.
9. Strictly decode and normalize proof nodes into witness vectors.
10. Build NonMembershipWitness with the exact original path.
11. Execute and/or prove.

The off-chain proving script MAY inspect the RPC-reported storage value as a readiness check, but the guest MUST prove MPT exclusion cryptographically.

12.4 Guest algorithm

1. Read NonMembershipWitness and require schema_version == 1.
2. Validate the witness structure using checked parsing.
3. Require proof_height > 0 and source_router != zero.
4. Require path.len() == 1 and validate path[0] exactly according to Section 10.
5. Set terminalPath = path[0] and require type byte == 0x02.
6. Compute pathDigest = keccak256(abi.encode(path)).
7. Compute hashedPath and logical storage slot K.
8. Verify source_router account inclusion under state_root.
9. Strictly decode the authenticated Router account and extract storageRoot S_H.
10. Compute secure storage-trie key keccak256(ABIEncode32(K)).
11. Verify the storage proof against S_H for that exact key.
12. Require the exact verifier result is Absent.
13. Reject Present(value) for every value, including zero.
14. Construct NonMembershipPublicValues.
15. ABI-encode and commit the public values.

12.5 Public values

schemaVersion
proofHeight H
stateRoot R_H
sourceRouter
pathDigest

There is no expected value in the non-membership output.

12.6 Solidity verifyNonMembership binding

The light client MUST:

1. Require caller == LOCAL_IBC_ROUTER.
2. Require msg_.proofHeight.revisionNumber == 0.
3. Let H = msg_.proofHeight.revisionHeight; load ConsensusState[H] and require it exists.
4. Require msg_.path.length == 1.
5. Strictly decode the canonical SP1ProofEnvelope from msg_.proof.
6. Strictly decode NonMembershipPublicValues and require schemaVersion == 1.
7. Require public.proofHeight == H.
8. Require public.stateRoot == stored.stateRoot.
9. Require public.sourceRouter == SOURCE_IBC_ROUTER.
10. Require public.pathDigest == keccak256(abi.encode(msg_.path)).
11. Verify the proof with NON_MEMBERSHIP_VKEY through SP1_VERIFIER.
12. Return uint256(stored.blockTimestamp).

12.7 Same-height timeout rule

Timeout safety MUST use receipt absence and authenticated time from the same consensus state:

ConsensusState[H] = {
  stateRoot: R_H,
  blockTimestamp: T_H,
  validatorSetHash: ...
}

non-membership proof:
  proves receipt absent under R_H

verification timestamp returned to Router:
  T_H from the same ConsensusState[H]

It is invalid to prove absence under R\_H1 and use a timestamp from H2.
The Router remains responsible for its normal timeout and packet-commitment checks. The light client is responsible only for proving absence under R\_H and returning the timestamp from that same consensus state.

13. SP1 on-chain verifier integration

13.1 Required dependency

contracts/lib/sp1-contracts MUST remain pinned to the release selected in Section 2. The light client uses the official upstream ISP1Verifier/SP1VerifierGateway and MUST NOT reimplement SP1 verifier cryptography.

13.2 Verifier address and proof mode

v1.1 retains the v1.0.1 EVM proof mode: Groth16 through the matching pinned SP1VerifierGateway route. Changing proof mode requires a specification revision and proof/verifier revalidation.

13.3 Program VKey isolation

The light client stores three immutable, nonzero, mutually distinct VKeys:

UPDATE_CLIENT_VKEY
MEMBERSHIP_VKEY
NON_MEMBERSHIP_VKEY

Mapping remains:

updateClient         -> UPDATE_CLIENT_VKEY
verifyMembership     -> MEMBERSHIP_VKEY
verifyNonMembership  -> NON_MEMBERSHIP_VKEY

The batched membership guest necessarily changes membership.elf and MEMBERSHIP_VKEY from v1.0.1.

The implementation SHOULD preserve update-client.elf/UPDATE_CLIENT_VKEY and non-membership.elf/NON_MEMBERSHIP_VKEY byte-for-byte when no source/dependency change requires regeneration. If they change, the release report MUST explain why.

13.4 Proof envelopes

updateClient and verifyNonMembership continue to use:

struct SP1ProofEnvelope {
    bytes publicValues;
    bytes proofBytes;
}

verifyMembership uses MembershipBatchProofEnvelope from Section 11.8.

The program VKey is never supplied by the relayer. Solidity chooses the immutable VKey according to the light-client method.

All opaque proof envelopes MUST be strictly ABI-decoded and canonical. Trailing bytes, malformed offsets, empty required fields, unsupported modes, or non-canonical re-encoding MUST be rejected.

13.5 Verified membership batch cache

The contract stores:

mapping(bytes32 batchId => bool verified) internal verifiedMembershipBatches;

A cache entry may be written only after:

- the stored ConsensusState[H] exists;
- membership public values match that stored state and configured Router;
- batchRoot and entryCount match the envelope; and
- ISP1Verifier verifies the proof under MEMBERSHIP_VKEY.

The cache is a performance optimization only. It does not create new trust.

13.6 Verification order

Cheap schema, mode, height, root, Router, count, path/value length, and Merkle-branch checks SHOULD occur before expensive external verification when the ordering does not permit acceptance of an unverified batch.

In FULL mode, state writes to verifiedMembershipBatches MUST occur only after the SP1 proof succeeds.

13.7 Deployment implication

Because the light-client bytecode, membership public-values schema, envelope decoder, and membership VKey change, v1.1 requires deployment of the updated BesuQBFTZKLightClient and standard IBC client registration/migration according to the pinned Router governance model.

This revision is not implementable by only replacing a VKey in an already deployed immutable v1.0.1 light-client instance.

14. BesuQBFTZKLightClient Solidity implementation

14.1 Contract purpose

BesuQBFTZKLightClient.sol implements the pinned upstream ILightClient interface and binds SP1 proof results to standard IBC Router operations. v1.1 adds batched membership verification behind the same verifyMembership ABI; upstream ICS26Router is unchanged.

14.2 Required imports

The contract MUST import, not duplicate:

- ILightClient and required IBC message/height types from the versioned interface-only bundle;
- ISP1Verifier from pinned sp1-contracts.

The project MAY add BatchMembershipLib for positional Merkle verification and strict envelope decoding.

14.3 Core state

Logical state:

address public immutable LOCAL_IBC_ROUTER;
address public immutable SOURCE_IBC_ROUTER;
address public immutable SP1_VERIFIER;

bytes32 public immutable UPDATE_CLIENT_VKEY;
bytes32 public immutable MEMBERSHIP_VKEY;
bytes32 public immutable NON_MEMBERSHIP_VKEY;

ClientState internal clientState;
mapping(uint64 => ConsensusState) internal consensusStates;
mapping(bytes32 => bool) internal verifiedMembershipBatches;

There are no internal clientId-keyed maps. Upstream ICS02 routing owns clientId -> ILightClient.

14.4 Constructor/initialization invariants

All v1.0.1 invariants remain:

LOCAL_IBC_ROUTER != 0
SOURCE_IBC_ROUTER != 0
SP1_VERIFIER != 0
all three VKeys != 0
all three VKeys pairwise distinct
initialHeight > 0
initialStateRoot != 0
initialValidatorSetHash != 0
initialBlockTimestamp > 0

Initialization stores exactly one bootstrap consensus state and an empty verifiedMembershipBatches cache.

14.5 Access boundary

updateClient, verifyMembership, and verifyNonMembership MUST require msg.sender == LOCAL_IBC_ROUTER when the pinned upstream call path is Router-mediated.

Public read-only getters remain callable by anyone.

14.6 Update-client behavior

Update-client state transition, duplicate/no-op behavior, conflict rejection, trusting-period rule, clock-drift rule, and consensus-state storage remain exactly as defined in Section 8.

14.7 Batched verifyMembership behavior

verifyMembership MUST implement Section 11.10 exactly.

It MUST NOT:

- trust a relayer-supplied storage slot;
- accept a CACHED envelope whose batchId was never verified;
- accept a leaf outside entryCount;
- accept a Merkle proof under another batchRoot;
- hash msg_.value again before leaf construction;
- write a batch-cache entry before SP1 verification;
- require any change to the upstream ICS26Router ABI.

14.8 verifyNonMembership behavior

Non-membership remains exactly as Section 12.6. It uses NON_MEMBERSHIP_VKEY and does not consult the membership batch cache.

14.9 misbehaviour(bytes) behavior

v1.1 does not define a misbehaviour proof format or fourth program. misbehaviour(bytes) MUST continue to fail closed with UnsupportedMisbehaviour().

14.10 Required getters

At minimum expose the upstream-required getClientState() plus the v1.0.1 getters for latest height, policy, consensus state, Router addresses, VKeys, and verifier.

Recommended additional getter:

function isMembershipBatchVerified(bytes32 batchId)
    external
    view
    returns (bool);

14.11 Events

The contract SHOULD emit:

ClientUpdated(height, stateRoot, timestamp, validatorSetHash)
MembershipBatchVerified(batchId, proofHeight, batchRoot, entryCount)

Events are observability outputs and are not security inputs.

14.12 State-growth note

Each successfully verified unique membership batch adds one boolean cache key. This is intentional to avoid repeated SP1 verification when batch items are delivered across one or more transactions.

Pruning verified batch IDs is not defined in v1.1. A future pruning feature MUST NOT allow an attacker to make an unverified CACHED envelope valid.

15. Proof-generation tooling and Proof API batching

The local Rust proving tooling remains responsible for RPC acquisition, witness construction, SP1 execution, and artifact generation. v1.1 additionally defines the batching/scheduling behavior expected from the production Proof API or equivalent coordinator.

Neither the Proof API nor the scheduler is a trust authority.

15.1 Required proving capabilities

The tooling MUST provide logical capabilities equivalent to:

execute update-client
prove update-client
execute membership-batch
prove membership-batch
execute non-membership
prove non-membership
build per-entry batch Merkle branches
encode FULL and CACHED membership envelopes

15.2 Update-client flow

The v1.0.1 update-client execute/prove flow remains unchanged.

For throughput orchestration, update jobs MUST be deduplicated by at least:

(route/light-client identity, targetHeight H)

If multiple pending batches require the same not-yet-stored H, only one active update proof job for that key SHOULD exist.

15.3 Membership proof intent

A membership proof intent represents one standard Router membership operation before SP1 proving begins.

Logical fields:

route/light-client identity
sourceRouter
path bytes[]
expectedCommitment bytes32
observedSourceHeight or event metadata
operation identifier / sequence for deduplication
timeout/deadline metadata when available

The intent MUST NOT carry an authoritative logical storage slot.

The same intent submitted repeatedly MUST be deduplicated before proving.

15.4 Cohort formation

The scheduler groups compatible membership intents that target the same source Router/light-client route.

Preferred defaults:

preferredBatchSize = 10
maxBatchSize = 32
maxBatchWaitMs = 3000

Sealing rules:

- seal immediately at maxBatchSize;
- if a prover slot is available and at least preferredBatchSize compatible intents are queued, the scheduler MAY seal early;
- otherwise seal when the oldest compatible intent reaches maxBatchWaitMs;
- deadline/timeout pressure MAY force an earlier smaller batch;
- a batch with one valid entry is allowed.

The scheduler MUST NOT wait indefinitely for a full batch.

15.5 Common proof-height selection

After a cohort is sealed, choose one latest eligible Besu height H such that:

- H is not lower than the source inclusion height needed by any selected intent;
- source block H is available from the configured RPC;
- historical eth_getProof at H is available;
- every selected commitment can be proven present at H;
- H can be accepted/reused by the destination light client under the existing trust policy.

H is then frozen for the batch.

If some selected entries are not present at H, split/requeue those entries; do not silently change H after witness generation begins.

15.6 Membership batch witness acquisition

For the frozen H:

1. fetch source block H;
2. obtain candidate R_H = header.stateRoot;
3. derive every K_i from the exact intent path;
4. call eth_getProof(sourceRouter, [K_1..K_N], hex(H));
5. normalize one accountProof and N storage proofs;
6. build the canonical sorted MembershipWitness;
7. SP1-execute/prove one membership batch;
8. build one Merkle branch per entry;
9. output one FULL envelope and N-1 CACHED envelopes for the preferred single-multicall flow.

If ConsensusState[H] already exists, the tooling MUST require its stored stateRoot equals R_H.

If it does not exist, the tooling MAY start update-client(H) and membership-batch(H) proving concurrently.

15.7 Parallel update and membership generation

allowParallelUpdateAndMembership SHOULD default to true, but actual worker concurrency is benchmark-gated.

The Proof API MUST NOT infer that 4 GPU workers imply 4 independent full-speed proof jobs. Operators must measure contention.

A safe scheduler model is:

membership batch job(H)  ----+
                             +--> wait for both artifacts --> build relay bundle
update job(T->H), if needed -+

On-chain transaction order remains:

updateClient(H), if needed
then recvPacket/ackPacket membership operations at H

15.8 Backpressure and job states

The scheduler MUST use a bounded queue. Incoming relayer events MUST NOT each create an immediately active prover job.

Recommended logical states:

QUEUED
BATCHED
HEIGHT_FROZEN
WITNESS_READY
PROVING
PROVED
SUBMITTED
CONFIRMED
FAILED_RETRYABLE
FAILED_TERMINAL

Exact persistence technology is not a security requirement. Production deployments SHOULD persist enough job metadata to recover from service restart without duplicate proof storms.

15.9 Relay artifact

The output for one membership cohort SHOULD contain:

proofHeight H
optional updateClient artifact
membership MembershipPublicValues
membership SP1 proof bytes
batchRoot
entryCount
per-entry leafIndex + Merkle branch
per-entry FULL/CACHED encoded proof bytes expected by verifyMembership

The first submitted item for a not-yet-cached batch SHOULD use FULL mode. Remaining items SHOULD use CACHED mode.

15.10 RPC response parsing

All fail-closed RPC parsing rules from v1.0.1 remain in force.

For multi-key eth_getProof additionally reject:

storageProof count != requested distinct key count
missing requested key
unexpected extra key
same returned key more than once
malformed per-key proof
accountProof disagreement across multiple fallback RPC calls

15.11 Proof artifact JSON

For development/testing, a deterministic JSON artifact MAY include:

{
  "schemaVersion": 2,
  "program": "membership",
  "proofHeight": 123,
  "batchRoot": "0x...",
  "entryCount": 10,
  "elfSha256": "0x...",
  "programVKey": "0x...",
  "publicValues": "0x...",
  "proofBytes": "0x...",
  "entries": [
    {"leafIndex": 0, "pathDigest": "0x...", "expectedCommitment": "0x...", "merkleProof": []}
  ]
}

This JSON is never an on-chain trust input.

15.12 Non-membership flow

The v1.0.1 single-item non-membership execute/prove flow remains unchanged. Timeout batching is outside v1.1.

15.13 Proof API external-change checklist

If the production Proof API lives outside this repository, the implementation completion report MUST identify these required external changes:

- proof-intent queue instead of immediate prove-on-request;
- deduplication by route/path/value and update-job key;
- cohort formation with preferred 10 / max 32;
- 2-5 second configurable coalescing window, default 3 seconds;
- common-height selection and freezing;
- multi-key eth_getProof witness acquisition;
- batch membership prover invocation;
- update/membership parallel job support;
- FULL/CACHED per-item envelope generation;
- relay bundle containing one optional update plus multiple packet/ack operations.

16. Error and rejection rules

Each deterministic Rust crate SHOULD expose typed errors. SP1 guests may terminate on invalid witness, but native execution/tests MUST preserve diagnostic categories.

Canonical categories:

Category

Examples

Schema/structure
  unsupported schema, malformed vector/node/path/header,
  invalid length encoding, integer/length overflow

QBFT encoding

malformed RLP, invalid extraData shape, invalid vote, invalid round

Validator set

empty, zero address, unsorted, duplicate, count overflow, hash mismatch

Signature

bad length, scalar out of range, bad recovery id, recovery failure, non-validator signer

Quorum

duplicate signer, insufficient target quorum, insufficient trusted overlap

Header

invalid optional-tail layout, target height <= trusted height

MPT

malformed node, bad reference, bad root, wrong path, account absent, invalid account RLP

Membership

empty batch, batch size > 32, duplicate pathDigest, non-canonical batch order, storage key absent, commitment mismatch, batch-root mismatch, invalid item Merkle branch, unknown cached batchId

Non-membership

requested storage key present, invalid exclusion proof

IBC path

path element count != 1, malformed terminal path, wrong path type, bad identifier, bad sequence width

Public binding

wrong schema, height, state root, Router, path digest, expected value

SP1

wrong VKey, malformed proof, verifier failure

Client state

missing trusted state, expired trusted state, future header, conflicting stored height

RPC/proving script

chainId mismatch, null block, archive data unavailable, malformed proof response

16.1 Fail-closed rule

No parser or verifier is allowed to turn an error into:

zero value
empty validator set
empty proof accepted as absence
default block height
default state root
ignored signer
ignored malformed node

unless this specification explicitly defines the empty representation as valid.

17. Rigorous unit-test specification

Testing is normative. A security-relevant function is not complete until its positive, negative, malformed-input, and boundary cases are covered. Tests MUST exercise deterministic library code natively and, where the same code is guest-compatible, through SP1 execution as specified later.

17.1 Test-data format

Repository test data MUST be immutable, reviewable, and self-describing. Every test-data directory MUST contain a manifest.json with:

testDataId
testDataVersion
sourceType              // synthetic | local-besu | upstream-vector
sourceReference         // block number/hash, command, or upstream test-vector name
inputFiles[]
expectedSuccess
expectedDecodedFields
expectedHashes
expectedPublicValuesHex // when applicable
expectedError           // when expectedSuccess=false

Real Besu test data MUST contain the raw JSON-RPC response exactly as collected plus a normalized deterministic representation used by tests. Synthetic invalid test data MUST document the single mutation applied to a known-good test vector.
No unit test may fetch the network. Network collection is performed only by explicit test-data collection scripts.

17.2 Canonical type and serialization tests

For every type in crates/types and crates/public-values, tests MUST cover:

encode -> decode -> exact equality
minimum legal values
maximum legal values
zero where legal
zero where forbidden
all fixed-byte lengths
truncated encoding
extra trailing bytes
unknown schema version
vector length 0 where forbidden
vector length 0 where forbidden
nested-vector length handling
malformed encoding
length overflow
fixed-byte length validation
integer endianness
address byte order
bytes32 preservation
nested-vector length handling
malformed ABI/public-value bytes

Golden public-value vectors MUST be checked by both Rust and Solidity.
Required golden structures:

UpdatePublicValues
MembershipPublicValues
NonMembershipPublicValues

For each golden vector:

Rust encode == test-data bytes
Rust decode(test-data bytes) == expected struct
Solidity decode(test-data bytes) == expected struct
Solidity re-encode(expected struct) == test-data bytes

17.3 QBFT extraData parser tests

crates/qbft/tests/extra\_data.rs MUST include valid cases for:

1 validator
4 validators
1 validator
4 validators
several validator counts covering different quorum boundary cases

It MUST reject:

empty extraData
missing vanity
vanity with wrong length
non-list root
wrong field count
missing validator list
malformed validator address
zero validator address
unsorted validators
duplicate validators
malformed vote
unknown vote type
malformed round
negative-equivalent/non-canonical round encoding
missing seals list
seal with wrong encoded length
malformed signature scalar
truncated RLP
non-canonical RLP
trailing data that changes the canonical object

Parser tests MUST compare accepted real test data with the validator addresses, vote, round, and seal count independently expected from the source block.

17.4 QBFT committed-seal signing-digest tests

crates/qbft/tests/signing\_digest.rs MUST contain at least one real Besu block test vector and MUST reproduce the digest expected from Besu's committed-seal algorithm.
The test MUST prove that committed seals are excluded while the QBFT round is retained.
Starting from one valid test vector, mutate each header field individually and assert that the digest changes and the original commit signatures no longer authenticate the mutated header. Mutations MUST cover every field supported by the selected Besu fork schema, including where present:

parentHash
ommersHash / sha3Uncles
beneficiary / coinbase
stateRoot
transactionsRoot
receiptsRoot
logsBloom
difficulty
number
gasLimit
gasUsed
timestamp
extraData vanity
extraData validators
extraData vote
extraData round
mixHash / prevRandao
nonce
baseFeePerGas
withdrawalsRoot
blobGasUsed
excessBlobGas
parentBeaconBlockRoot
requestsHash
blockAccessListHash

Changing only the embedded commit-seal bytes MUST NOT change the reconstructed committed-seal digest because the signing preimage uses an empty commit-seal list.
A separate test MUST demonstrate that the committed-seal digest is not accidentally replaced by Besu's on-chain BFT block-reference hash that also removes/zeros the round.

17.5 RLP tests

The strict RLP implementation used by header and MPT code MUST test:

empty byte string
single byte < 0x80
single byte >= 0x80
short string
55-byte string
56-byte string
long string
empty list
short list
55-byte list payload
56-byte list payload
nested lists
malformed length declaration that would overflow checked arithmetic

It MUST reject non-canonical or malformed encodings including:

single byte encoded as a longer string
leading-zero length-of-length
length-of-length containing zero prefix
long form used when short form is required
payload shorter than declared length
payload longer than declared length
integer with forbidden leading zero
truncated nested list
integer overflow
length overflow
trailing bytes where exact consumption is required

17.6 Keccak and SHA-256 tests

Cryptographic hash wrappers MUST use known public vectors and protocol-specific vectors.
At minimum test:

Keccak256(empty)
Keccak256("abc")
SHA256(empty)
SHA256("abc")
Ethereum account secure-trie key
Ethereum storage secure-trie key
validatorSetHash vector
IBC terminal path hash
IBC logical storage slot K
pathDigest

Tests MUST explicitly distinguish Keccak-256 from standardized SHA3-256.

17.7 secp256k1 / commit-seal tests

crates/qbft/tests/signatures.rs MUST validate the full pipeline:

(committed-seal digest, r, s, recovery-id)
    -> recover public key
    -> verify ECDSA relation
    -> derive Ethereum address

Positive vectors MUST include multiple real committers from at least one real Besu QBFT header.
Negative vectors MUST include:

r = 0
s = 0
r >= curve order
s >= curve order
invalid recovery id
wrong signature length
mutated r
mutated s
mutated digest
wrong recovered key
invalid encoded signature
non-validator signer
signature valid for another header

If the selected Besu release applies any canonical-S/low-S restriction to this signature path, the implementation and tests MUST mirror that exact selected-release rule. If it does not, the implementation MUST NOT invent one.

17.8 Validator-set normalization and commitment tests

crates/qbft/tests/validators.rs MUST test:

one validator
multiple validators
strictly ascending
reordered set rejected
duplicate rejected
zero rejected
empty rejected
validator count conversion to the U32_BE commitment count is checked

Golden vectors MUST test the exact commitment:

Keccak256(
  ASCII("BESU_QBFT_VALIDATOR_SET_V1")
  || U32_BE(count)
  || validator[0]_raw20
  || ...
)

Changing count, order, or any address byte MUST change the commitment.

17.9 Duplicate-committer tests

The same validator MUST never increase a quorum count twice.
Required cases:

A,B,C         -> 3 unique committers
A,A,B         -> reject
A,A,A         -> reject
same signer with byte-different valid ECDSA encodings -> reject by recovered address
same signer repeated at non-adjacent positions -> reject

The implementation MUST reject a duplicated signer rather than merely de-duplicate and continue.

17.10 Target QBFT quorum boundary tests
Tests MUST cover validator counts from every N mod 3 class, including
small boundary values and arithmetic edge cases, and MUST prove the
integer implementation exactly matches the normative formula without
floating-point arithmetic.

targetQuorum(N) = ceil(2*N/3)

The test MUST assert:

q-1 unique target validators -> fail, when q > 0
q   unique target validators -> pass
q+1 unique target validators -> pass, when <= N

The test MUST cross-check the integer function against the formula without floating-point arithmetic.

17.11 Trusted-set overlap boundary tests

Tests MUST cover validator counts from every N mod 3 class, including
small boundary values and arithmetic edge cases, and MUST prove the
integer implementation exactly matches the normative formula without
floating-point arithmetic.

trustedThreshold(N) = floor(N/3) + 1

The test MUST assert:

t-1 distinct recovered signers belonging to V_T -> fail, when t > 0
t   distinct recovered signers belonging to V_T -> pass
t+1 distinct recovered signers belonging to V_T -> pass, when possible

The trusted count and target count MUST be computed independently over the same unique recovered-signer set.

17.12 Trusting-period and clock-drift tests

If trusting\_period\_seconds > 0, test:

trustedTimestamp + trustingPeriod > now  -> pass
trustedTimestamp + trustingPeriod == now -> fail
trustedTimestamp + trustingPeriod < now  -> fail

For target time:

targetTimestamp <= now + maxClockDrift -> pass
targetTimestamp == now + maxClockDrift -> pass
targetTimestamp > now + maxClockDrift  -> fail

Overflow-safe arithmetic MUST be tested near u64::MAX. Tests MUST not depend on wall-clock time; inject a deterministic now into native verification and use the contract test VM clock on Solidity.

17.13 Supported Besu block-header encoding tests

crates/qbft/tests/header_rlp.rs MUST validate every Besu block-header schema accepted by the implementation. The purpose of these tests is to prove that the update-client reconstructs the exact canonical header RLP used for the QBFT committed-seal signing digest.


At minimum, committed repository test data MUST include:

- one real QBFT block header generated by the controlled local Besu 26.5.0
  development network;
- exact reconstruction of the committed-seal signing digest for that header;
- deterministic header-schema vectors covering every Besu header format
  supported by v1.1.

Private or production Besu deployment headers MAY be used separately for
controlled compatibility testing when explicitly approved, but they MUST
NOT be required as committed reusable repository test data.

Tests MUST also ensure the optional header-tail ordering implemented by Section 6.2 remains canonical:

pre-London-like tail cannot contain baseFee
post-London-like tail encodes baseFee at the correct position
withdrawalsRoot appears only in the correct schema position
blobGasUsed/excessBlobGas ordering is correct
parentBeaconBlockRoot ordering is correct when enabled
requestsHash ordering is correct when enabled
blockAccessListHash follows requestsHash when enabled
unsupported optional-field combinations are rejected

These are Besu execution block-header encoding tests; they do not implement Ethereum PoS consensus. The optional fields are part of the signed QBFT header and therefore affect the committed-seal digest.

The guest supports only the ordered optional tail fields defined in Section 6.2. The off-chain proving script MUST reject an RPC header field that would extend the canonical RLP header beyond this supported list. Adding a new proof-relevant header field requires a specification/program update.

17.14 Ethereum account-MPT inclusion tests

crates/evm-mpt/tests/account\_inclusion.rs MUST cover valid proofs involving:

branch -> leaf
branch -> extension -> leaf
extension -> branch -> leaf
multiple extensions
hashed child references
inline child references

A valid Router account proof MUST decode the account RLP and expose exactly:

nonce
balance
storageRoot
codeHash

Negative tests MUST cover:

wrong stateRoot
wrong account address
wrong secure-trie key
wrong nibble path
wrong branch child
wrong extension prefix
wrong leaf suffix
mutated RLP
mutated child hash
missing required node
malformed node
truncated proof
proof for a neighboring account
root mismatch
account absent

Membership/non-membership programs MUST reject an absent Router account.

17.15 Ethereum storage-MPT membership tests

crates/evm-mpt/tests/storage\_inclusion.rs MUST use both synthetic tries and real eth\_getProof test data.
Required positive assertion:

verify_storage(storageRoot, K, proof) == Present(expectedValue)

Negative cases:

wrong storageRoot
wrong logical slot K
wrong secure-trie key
wrong value
mutated branch
mutated extension
mutated leaf
mutated inline child
mutated hashed child
truncated proof
extra node used to hide malformed traversal
proof for neighboring present key

17.16 Ethereum storage-MPT non-membership tests

crates/evm-mpt/tests/storage\_exclusion.rs MUST cover all exclusion forms handled by the verifier:

missing branch child
extension-path divergence
leaf-path divergence
empty trie/root case when applicable

Negative cases MUST include:

requested key actually present
proof for another absent key
wrong root
wrong logical slot
truncated exclusion proof
malformed compact path
proof terminates before cryptographic exclusion is established

A crucial regression test MUST construct a present storage leaf whose decoded value is zero and assert:

Present(0) != Absent

The non-membership guest MUST accept only the Absent result.

17.17 Nibble and hex-prefix tests

crates/evm-mpt/tests/hex\_prefix.rs MUST test:

odd leaf path
even leaf path
odd extension path
even extension path
empty remainder where protocol-valid
byte-to-nibble round trip

It MUST reject:

invalid flag nibble
invalid padding nibble
overlong/non-canonical compact representation
path longer than 64 nibbles for a 32-byte secure key

17.18 IBC path tests

crates/ibc-storage/tests/paths.rs MUST provide golden vectors for packet, receipt, and acknowledgement paths:

packet          = clientId || 0x01 || U64_BE(sequence)
receipt         = clientId || 0x02 || U64_BE(sequence)
acknowledgement = clientId || 0x03 || U64_BE(sequence)

Test sequence boundaries:

0, if accepted by the pinned upstream call surface
1
2^32
2^64 - 1

The exact bytes[] Router-facing shape MUST be tested:

valid:   path.length == 1 and path[0] == terminalPath
invalid: path.length == 0
invalid: path.length > 1

Reject:

terminal path too short
invalid client identifier under pinned upstream rules
wrong type
wrong sequence width
trailing bytes
empty client identifier

For every vector, keccak256(abi.encode(path)) computed by Rust MUST equal the Foundry/Solidity reference digest.

17.19 Router logical-slot tests

crates/ibc-storage/tests/storage\_slots.rs MUST test:

hashedPath = Keccak256(terminalPath)
K = Keccak256(ABIEncode32(hashedPath) || ABIEncode32(IBCSTORE_STORAGE_SLOT))

For every golden vector:

Rust native K == SP1 guest K == Solidity/Foundry reference K

Mutating the path, type byte, sequence, mapping namespace, or 32-byte ABI padding MUST change K and MUST fail the corresponding proof test vector.

17.20 Packet and acknowledgement expected-commitment binding tests

The Besu ZK code does not implement a second packet/acknowledgement commitment algorithm. Tests MUST instead use packet and acknowledgement examples from the pinned upstream IBC contracts implementation and record the exact 32-byte expected commitment supplied to verifyMembership.

For each packet and acknowledgement test vector, the test MUST establish:

upstream IBC operation provides exact expected 32-byte commitment
        +
path-derived Router storage proof authenticates the same 32-byte word
        |
        v
membership proof succeeds

Negative tests MUST independently mutate:

expected commitment byte
IBC path/type byte
sequence
Router storage proof/value

and require the membership proof or Solidity binding to fail as appropriate.

The test MUST also assert that each exact 32-byte expected commitment is passed into MembershipEntryWitness.expected_commitment unchanged, is authenticated against Router storage, and is used directly in the batch leaf. The Besu ZK implementation MUST NOT apply the upstream packet/ACK commitment hash again.

17.21 update-client program unit tests

Positive test:

stored trusted hash corresponds to V_T
valid target header H
valid target validator set V_H
valid unique target commit seals
trusted overlap reached
target quorum reached
valid time rules

Negative tests MUST independently exercise:

wrong trusted validator-set hash
fake V_T with otherwise valid signatures
empty V_T
unsorted V_T
duplicate V_T
zero validator in V_T
H <= T
malformed target extraData
empty V_H
unsorted V_H
duplicate V_H
invalid commit seal
signature for another header
duplicate committer
committer outside V_H
insufficient target quorum
insufficient trusted overlap
unsupported header schema

Golden test MUST assert the exact UpdatePublicValues bytes.

17.22 Membership-program unit tests

Positive cases MUST cover batch sizes:

1
5
10
20
32

Every positive batch MUST prove real or synthetic Router packet/ack commitments under one shared Router account proof.

Negative cases MUST include:

empty entries
33 entries
wrong stateRoot
wrong Router address
wrong path element count
wrong path bytes
wrong terminal path type
wrong sequence
wrong derived K
wrong shared account proof
wrong Router storageRoot
wrong per-entry storage proof
storage key absent
actual commitment != expected commitment
duplicate pathDigest
non-canonical entry ordering
packet commitment passed for an ACK path
ACK commitment passed for a packet path
one mutated entry among otherwise valid batch
one missing storage proof
one extra storage proof
malformed membership schema

Golden tests MUST assert exact MembershipPublicValues bytes and exact batchRoot for fixed batch vectors.

17.22.1 Batch-Merkle deterministic tests

Rust and Solidity MUST share golden vectors for:

N = 1, 2, 3, 5, 10, 20, 32
EMPTY_BATCH_LEAF
odd/non-power-of-two padding
leaf mutation
left/right branch ordering
invalid leaf index
invalid branch length
wrong batchRoot

A proof valid for one index MUST fail for another index unless the leaf is actually identical, and duplicate pathDigest entries are forbidden before tree construction.

17.23 Non-membership-program unit tests

Positive case MUST prove exact receipt-slot absence.
Negative cases:

receipt actually present
proof of absence for another receipt path
wrong source Router
wrong stateRoot
wrong path element count
wrong path type
wrong sequence
wrong K
Router account absent
malformed exclusion proof
Present(0) incorrectly offered as absence

Golden test MUST assert exact NonMembershipPublicValues bytes.

17.24 RPC adapter, witness-builder, and scheduler tests

The off-chain tooling is not a trust boundary, but it MUST fail deterministically and MUST not silently default malformed fields.

Retain all v1.0.1 eth_getBlockByNumber and eth_call parsing tests.

For multi-key eth_getProof, test:

valid 1-key response
valid 10-key response
valid 32-key response
wrong returned address
empty accountProof
missing storageProof
returned key not requested
same key returned twice
missing one requested key
unexpected extra key
RPC response order different from request order and explicit key matching restores canonical order
malformed proof-node hex
malformed node length / RLP length overflow
RPC error
historical state unavailable

Witness builders MUST be deterministic: the same normalized RPC data and intents produce byte-identical canonical MembershipWitness encoding and batchRoot.

Scheduler tests MUST cover:

intent deduplication
preferred batch size 10
hard cap 32
max wait expiry
small batch after wait expiry
height frozen once witness collection begins
one update job for multiple batches requiring same H
ConsensusState[H] exists -> update proof omitted
membership and update job creation can proceed concurrently
stale/absent entry split or requeue
bounded queue/backpressure

17.26 Solidity codec and binding unit tests

PublicValuesCodec.t.sol MUST cover exact decoding and rejection of:

valid update public values
valid membership schema-v2 public values
valid non-membership public values
wrong schema version
truncated bytes
trailing bytes
wrong field width

MembershipBatchProofEnvelope tests MUST cover:

valid FULL
valid CACHED
unsupported mode
FULL with empty proof/public values
CACHED with non-empty proof/public values
entryCount 0
entryCount 33
leafIndex >= entryCount
invalid Merkle branch
unknown cached batchId
batchRoot mismatch

The Solidity/Rust binding tests MUST cross-check:

Keccak256(abi.encode(bytes[] path))
all Section 6.8 leaf/node vectors
batchId computation

17.27 Unit-test completion gate

A PR changing any security-critical deterministic function MUST include or update tests that prove:

1. intended valid input still passes;
2. direct invalid input fails;
3. boundary behavior is explicit;
4. a one-field mutation cannot bypass the check.

Line-coverage percentage alone is not an acceptance criterion. Every explicit rejection branch in Sections 8-14 MUST have at least one named test that reaches it.



19. SP1 execution and proof-generation tests

19.1 Native/SP1 differential execution

The deterministic libraries used by each guest MUST first be runnable natively. Every golden proof test vector MUST be evaluated in both modes:

native verifier execution
        ==
SP1 execute mode public values / rejection

The outcome and public-value bytes MUST match exactly.

19.2 Execute-mode suite

Each guest MUST have SP1 execute-mode tests for:

update-client: valid + every major rejection class
membership: valid batches of 1, 5, 10, 20, and 32 + every major rejection class
non-membership: valid batches of 1, 5, 10, 20, and 32 + every major rejection class

A security failure MUST occur inside the guest even when the off-chain pre-check is disabled.

19.3 Real proof-generation suite

For update-client and non-membership, one golden valid vector per program is sufficient. For membership, CI release jobs MUST include at least batch size 1 and batch size 10, and release benchmarking MUST additionally execute 5, 20, and 32. For each required real-proof vector, CI release jobs MUST:

1. build the ELF;
2. setup proving/verifying keys;
3. generate a Groth16 EVM proof using the pinned SP1 v6.1.0 SDK;
4. verify the proof locally with the SP1 SDK;
5. verify that the proof is accepted through the configured upstream
   Groth16 verifier/SP1VerifierGateway route;
6. extract public values;
7. require exact equality with the golden public-value bytes.

19.4 Program-VKey isolation

The three guest ELFs MUST have three different program VKeys. Tests MUST assert:

update proof + UPDATE_CLIENT_VKEY       -> valid
update proof + MEMBERSHIP_VKEY          -> invalid
update proof + NON_MEMBERSHIP_VKEY      -> invalid

membership proof + MEMBERSHIP_VKEY      -> valid
membership proof + UPDATE_CLIENT_VKEY   -> invalid
membership proof + NON_MEMBERSHIP_VKEY  -> invalid

nonmembership proof + NON_MEMBERSHIP_VKEY -> valid
nonmembership proof + UPDATE_CLIENT_VKEY  -> invalid
nonmembership proof + MEMBERSHIP_VKEY     -> invalid

19.5 Proof/public-value tamper tests

For each valid proof artifact:

mutate one public-value byte -> verification/binding fails
truncate public values -> fail
append public-value bytes -> strict decoder fails
mutate proof bytes -> verifier fails
mutate batchRoot or entryCount in membership public values -> binding fails
mutate one per-item Merkle branch -> item binding fails
truncate proof bytes -> verifier/gateway fails
replace proof with another program's proof -> fail

20. Solidity and SP1 verifier integration tests

These tests verify the boundary between the three real SP1 programs and BesuQBFTZKLightClient.sol. They are not the same as the two-chain E2E tests in Section 21.

The real-proof integration flow is:

recorded/local test data
        -> build typed witness
        -> generate real SP1 proof + publicValues
        -> deploy/use real ISP1Verifier or compatible gateway
        -> deploy BesuQBFTZKLightClient with the matching program VKeys
        -> submit the proof through the corresponding light-client function
        -> assert accepted state/return value

A valid proof is then mutated or rebound to the wrong context to prove that the Solidity/SP1 boundary fails closed.

The target Besu networks are configured as gas-free/zero-fee networks. These integration tests therefore assert proof validity, state changes, return values, and rejection behavior; they do not use transaction fee cost as a correctness criterion.

20.1 Mock-verifier policy

A mock verifier MAY be used only for isolated Solidity state-machine and public-binding unit tests. It MUST NOT be used as evidence that SP1 integration works.

At least one release/conformance suite MUST deploy or use the real ISP1Verifier implementation/gateway compatible with the pinned SP1 proof output and submit real proofs generated by the three programs.

The recommended development split is:

MockSP1Verifier
  -> fast tests for Solidity state transitions and public-value binding

Real ISP1Verifier / gateway
  -> conformance test proving actual SP1 proofBytes + publicValues are accepted

20.2 updateClient contract tests

Required cases:

valid proof stores ConsensusState[H]
valid proof advances latestHeight
wrong UPDATE_CLIENT_VKEY cannot be supplied/selected
invalid SP1 proof fails
unknown trustedHeight fails
trusted validator hash mismatch fails
H <= trustedHeight fails according to Section 14
conflicting state at an already stored height reverts and does not overwrite history
identical re-submission follows the explicitly specified idempotency rule
wrong schema fails
expired trust fails
future timestamp fails

Storage assertions MUST confirm that only:

stateRoot
blockTimestamp
validatorSetHash

are stored per consensus height; no full validator array may be persisted.

For the real-SP1 positive case, the test MUST generate an update proof from the canonical update-client ELF, submit its exact proofBytes and publicValues, and verify that the state stored at H exactly matches the public values authenticated by the proof.

20.3 verifyMembership contract tests

Required cases:

FULL batch proof verifies once and returns stored consensus timestamp
FULL verification caches batchId
CACHED item from same batch succeeds without another SP1 verifier call
unknown proofHeight fails
wrong stateRoot fails in FULL mode
wrong source Router fails in FULL mode
wrong batchRoot fails
wrong entryCount fails
entryCount 0 / 33 fails
leafIndex out of range fails
wrong Router-supplied path fails item Merkle verification
wrong Router-supplied expected commitment fails item Merkle verification
invalid positional branch fails
unknown cached batchId fails
CACHED envelope containing proof/public values fails canonical mode rules
membership proof bytes under wrong VKey fail
non-membership proof submitted to FULL membership path fails via VKey separation

The real-SP1 positive test MUST generate one batch proof containing multiple authenticated Router storage entries. The first verifyMembership call uses FULL mode; at least one subsequent call uses CACHED mode and must be accepted through a per-item Merkle branch.

A test MUST instrument/mock the verifier call count and prove that one batch of N item verifications invokes the SP1 verifier exactly once after the batch is cached.

20.4 verifyNonMembership contract tests

Required cases:

valid proof returns stored consensus timestamp
unknown proofHeight fails
wrong stateRoot fails
wrong Router/path digest fails
membership proof cannot be used
malformed public values fail

For the real-SP1 positive case, the proof MUST establish cryptographic absence of the exact receipt storage key under the same state root stored in ConsensusState[H].

20.5 Timeout same-state tests

The integration harness MUST demonstrate that timeout logic uses the timestamp stored in the same ConsensusState[H] whose root is bound by the non-membership proof.

Required negative test:

non-membership proof at H1
+
timestamp from newer H2

MUST NOT be accepted as one timeout state.

Required application-layer assertions:

receipt absent + timeout not reached -> timeout fails
receipt absent + timeout reached      -> timeout may proceed
receipt present                       -> non-membership proof fails

20.6 Real SP1VerifierGateway tests

The Foundry integration suite MUST test:

correct ISP1Verifier address
correct proof selector/gateway route
correct program VKey
correct public values
correct proof bytes

Negative cases MUST include a missing/frozen gateway route where the gateway is used, wrong verifier selector, wrong program VKey, malformed proof bytes, and a proof/public-value pair from the wrong SP1 program.

A successful real-verifier test is the conformance evidence that the Rust/SP1 proof artifact format, public-value ABI encoding, selected program VKey, upstream verifier contract, and Solidity light-client binding all agree.

21. Two-Besu-chain end-to-end conformance tests

21.1 Test topology

e2e/docker-compose.yml MUST start two independent generic Besu QBFT development networks.
Default conformance topology:

Chain A: 4 QBFT validators
Chain B: 4 QBFT validators

The local development networks SHOULD use the same zero-fee/gas-free style required by the target environments (for example the appropriate Besu zero-base-fee genesis configuration). Fee accounting is not an E2E correctness assertion.

Validator keys MUST be generated at E2E startup into ignored .e2e-state/ directories. No reusable production key or organization-specific key/address may exist in the repository.
Each chain MUST deploy:

pinned ICS26Router stack required by the selected IBC release
SP1VerifierGateway and matching Groth16 verifier required by the pinned SP1 stack
BesuQBFTZKLightClient that verifies the opposite chain
a minimal test IBC application sufficient to send/receive packets and ACKs

21.2 Bootstrap

The E2E setup MUST:

1. wait for both chains to produce finalized QBFT blocks;
2. select bootstrap height T on each source;
3. decode V_T from source QBFT extraData;
4. calculate validatorSetHash_T with the normative algorithm;
5. initialize the opposite light client with T, stateRoot_T, timestamp_T, validatorSetHash_T;
6. register the client/counterparty using the pinned IBC contracts interface.

The full validator set MUST NOT be written into the light-client contract.

21.3 Required positive E2E scenarios

The suite MUST exercise in both directions where applicable:

updateClient from T to later H using one target header
packet commitment membership and recvPacket
acknowledgement membership and ackPacket
receipt non-membership and timeout
reuse of an already stored ConsensusState[H] without another updateClient
multiple membership proofs using the same stored H
one batched membership proof containing 10 packet commitments under one H
one batched acknowledgement membership proof containing multiple ACK commitments
one updateClient(H) followed by multiple recvPacket calls in one multicall
one already-stored H with membership batch and no updateClient call
FULL first membership envelope followed by CACHED envelopes for the same batch
a later update using hash-stored validator authority plus private V_H witness

21.4 Required negative E2E scenarios

Inject and reject:

fake relayer proof bytes
wrong target block/header
wrong proof height
wrong stored root
wrong source Router
wrong IBC path
wrong packet bytes / commitment
wrong ACK bytes / commitment
receipt proof for wrong sequence
old proof replayed in a mismatched context
cached batchId replayed with different H/root/path/value
invalid item Merkle branch within an otherwise valid batch
valid proof from the opposite program VKey
modified public values
insufficient trusted overlap test data
insufficient target quorum test data
duplicate commit-seal test data
conflicting update for an already stored height

21.5 Historical proof availability

E2E MUST query eth\_getProof at non-head historical heights used by a stored consensus state. If the configured Besu node cannot serve the required historical proof, the test MUST fail with an explicit archive/history-availability error rather than falling back to latest state.

22. Performance and resource gates

22.1 Mandatory measurements

For each SP1 program, release benchmarking MUST record:

SP1 cycle count
guest peak memory, where tooling exposes it
proving-process peak RSS
serialized witness size
proof-generation wall-clock time
proof byte size
CPU utilization
GPU utilization when available

For membership, measurements MUST additionally record:

entry count
seconds per batch
messages per second
messages per hour
incremental cycles per additional membership entry
shared account-proof size
sum of per-entry storage-proof sizes

22.2 Required membership benchmark matrix

Benchmark the same valid route/profile with batch sizes:

1
5
10
20
32

The exact same benchmark hardware/backend MUST be used when comparing batch sizes in one report.

The first production tuning decision SHOULD choose the batch size that maximizes sustained messages/second while remaining within memory and queue-latency limits. preferredBatchSize=10 is only the initial operating default.

22.3 Current large-VM reference baseline

The current pre-batching reference environment is:

CPU: 96 vCPU
RAM: 320 GB
GPU workers: 4
GPU VRAM: 24 GB each
CPU workers: 1

Measured 3-run averages supplied for the current implementation:

4-validator update-client: 105.420 s
packet membership: 85.929 s
forward proof total: 191.348 s
1-validator update-client: 87.302 s
acknowledgement membership: 83.886 s
acknowledgement proof total: 171.188 s
all four proofs: 362.536 s

These values are a regression/reference baseline only. They are not a promised SLA for the batched design.

22.4 Throughput acceptance metric

For batch size N, report at minimum:

membershipThroughput(N) = N / membershipBatchProofSeconds(N)

For a full packet + acknowledgement benchmark where both directions batch the same N, report the measured end-to-end proof throughput from actual scheduler/prover execution rather than extrapolating ideal GPU parallelism.

A batched implementation MUST NOT be described as a throughput improvement unless the measured membership throughput at N > 1 exceeds the measured single-entry membership throughput on the same hardware/backend.

22.5 Parallel-proof benchmark

Benchmark at least:

A. update-client alone
B. membership batch alone
C. update-client + membership batch started concurrently

Record slowdown/contended resource use. The scheduler's max active-job configuration MUST be based on these measurements, not GPU-worker count alone.

22.6 No invented hardware SLA

v1.1 does not define a universal proof-time SLA. Proof time depends on SP1 backend, worker allocation, batch size, witness shape, and hardware. The release manifest MUST identify benchmark hardware/backend.

22.7 Regression baseline

Retain the v1.0.1 deterministic regression gate: later releases require explicit review when SP1 cycles, serialized witness size, or proof size increase by more than 20% for the same benchmark vector/class.

Membership batch-size-specific baselines MUST be versioned separately so a size-10 regression is compared with the previous size-10 benchmark, not with size 1.

23. Reproducible build, ELF, and VKey requirements

23.1 Pinned build inputs

A release MUST pin:

Rust toolchain
SP1 release
guest build toolchain/image
Cargo.lock
all git dependencies by immutable revision/tag
Solidity compiler
Foundry version
ibc-contracts commit
sp1-contracts commit/tag
selected Besu release used for conformance test data

No release artifact may depend on an unpinned Git branch such as main.

23.2 ELF reproducibility

scripts/check-reproducible-build.sh MUST:

1. build each guest from a clean environment;
2. record SHA-256 of each ELF;
3. destroy build output;
4. build again from the same pinned inputs;
5. require identical ELF hashes.

Required ELFs:

update-client.elf
membership.elf
non-membership.elf

23.3 VKey reproducibility

scripts/generate-vkeys.sh MUST run SP1 setup on each ELF and output the program VKey.
The generated artifacts/vkeys.json and the release manifest MUST record the following identities:

{
  "update-client": {"elfSha256":"0x...","programVKey":"0x..."},
  "membership": {"elfSha256":"0x...","programVKey":"0x..."},
  "non-membership": {"elfSha256":"0x...","programVKey":"0x..."}
}

Release manifest generation MUST fail if an ELF hash differs from its generated artifacts/vkeys.json entry.
Reviewers MUST compare the generated identities with the previous reviewed release and record changes
in the release manifest and deployment/migration plan. Historical expected VKeys are not an automatic
build or deployment gate.

23.4 Contract reproducibility

Release artifacts MUST record:

light-client deployed bytecode hash
light-client creation bytecode hash
ABI hash
Solidity compiler settings
SP1 verifier/gateway dependency identity

24. Deployment and release manifest

Runtime RPC URLs and deployed addresses are supplied to the proving/deployment scripts through CLI arguments and/or environment variables as described in Section 4. The repository does not require a canonical deployment-configuration YAML/TOML file.

Private RPC credentials, validator keys, and other secrets MUST NOT be committed.

24.1 Release manifest

Every tagged release MUST publish a machine-readable manifest containing at least:

spec document ID/version
Git commit of this repository
Rust toolchain
SP1 version
sp1-contracts tag/commit
ibc-contracts tag/commit
selected Besu conformance release(s)
Solidity compiler + optimizer configuration
Foundry version
three ELF SHA-256 hashes
three SP1 program VKeys
light-client ABI hash
light-client bytecode hash
public-values schema versions, including membership schema v2
MAX_BATCH_MESSAGES
membership batch leaf/node domain bytes and EMPTY_BATCH_LEAF
preferred benchmark batch size used for the release
validator-set hash domain string
IBC storage namespace constant
path type-byte constants
performance baseline reference

The release manifest MUST NOT contain RPC credentials, private keys, or private deployment addresses that are not intended to be public.

24.2 Deployment freeze checks

Before deploying a production instance, scripts MUST verify:

configured source uses QBFT header-validator mode
source chainId matches expected operational configuration
source Router address is non-zero
source Router code exists
pinned IBC storage-layout vectors match the compiled upstream dependency
SP1VerifierGateway exists and the required Groth16 verifier route is configured
program VKeys match release/expected-vkeys.json
membership VKey matches the schema-v2 batched membership ELF
light-client bytecode includes the verified-membership-batch cache/envelope decoder
bootstrap state fields are non-zero where required
bootstrap validator-set hash was produced by the normative hash algorithm
trusting period and clock-drift values are explicitly supplied

A deployment using missing/placeholder runtime values MUST be rejected.

25. Coding-agent implementation rules

Codex or any other coding agent implementing this specification MUST follow these rules.

Read this complete file before modifying code. Do not implement from headings alone.

Preserve exactly three SP1 programs: update-client, membership, and non-membership.

Do not add sequential T+1 ... H header replay.

Do not store validator arrays on-chain.

Do not support contract-based Besu validator selection in v1.1.

Do not trust RPC responses, relayer requests, Proof API queue contents, scheduler decisions, or prover-supplied storage slots.

Do not let the Proof API supply an authoritative storage slot K. Every membership entry derives K independently inside the guest.

Do not accept RPC zero as non-membership.

Do not combine membership and non-membership into one proof program.

Do not let the relayer choose a program VKey.

Do not fork the pinned ICS26Router. The throughput design MUST fit behind standard updateClient / verifyMembership / verifyNonMembership and Router multicall.

Do not create one membership SP1 proof per relayer event when multiple compatible intents are available for a batch.

Do not verify the Router account MPT separately for every entry inside one membership batch. Verify it once, then verify N storage proofs under the authenticated storageRoot.

Do not omit canonical batch ordering, duplicate rejection, batchRoot derivation, or per-item Merkle verification.

Do not accept a CACHED membership envelope before its batchId was authenticated by a successful FULL SP1 verification.

Do not assume 4 GPU workers provide 4x independent-job throughput. Implement concurrency as runtime configuration and benchmark it.

If the production Proof API is outside this repository, do not fabricate another production service here. Implement the integration boundary and report EXTERNAL_CHANGE_REQUIRED items from Section 15.13.

Add corresponding unit/negative tests in the same change as every security-relevant branch.

If a necessary behavior is undefined or conflicts with pinned upstream source, stop with BLOCKED_ON_SPEC and report the exact conflict.

25.1 Required implementation order

Codex SHOULD implement in this order:

1. inspect current v1.0.1 implementation and preserve unchanged update/non-membership behavior
2. add batch-merkle deterministic Rust library + golden vectors
3. update canonical membership witness/public-value types to schema v2
4. update membership guest for shared account proof + 1..32 storage proofs
5. update prover tooling for multi-key eth_getProof and canonical batch witness generation
6. generate per-entry Merkle branches and FULL/CACHED envelopes
7. update BesuZKTypes/PublicValuesCodec
8. update BesuQBFTZKLightClient batch cache and verifyMembership binding
9. add mock-verifier Solidity batch tests
10. add real SP1 batch proof tests
11. add proof-api-integration batching/scheduler boundary
12. add two-Besu-chain batched packet and ACK E2E tests
13. run reproducibility and regenerate membership ELF/VKey
14. benchmark membership sizes 1, 5, 10, 20, 32
15. benchmark update+membership concurrency
16. update release manifest and migration/deployment instructions

25.2 Required completion report

The coding agent MUST report:

files created/changed
exact pinned dependency versions
three ELF hashes
three program VKeys
whether update/non-membership ELF/VKeys remained unchanged from v1.0.1
membership schema version
MAX_BATCH_MESSAGES
unit-test counts/results
SP1 execute/proof results for batch sizes 1/5/10/20/32
Solidity tests/results including verifier-call-count cache test
E2E results
batch performance measurements
parallel update+membership benchmark results if available
light-client deployment/migration changes
EXTERNAL_CHANGE_REQUIRED Proof API items, if the service lives elsewhere
any BLOCKED_ON_SPEC items

It MUST NOT claim completion while a required acceptance item in Section 26 is unchecked.

26. Final implementation acceptance checklist

The repository is conformant only when all applicable items below are satisfied.

26.1 Architecture and scope

[ ] Generic Besu QBFT <-> Besu QBFT only; no private organization/deployment details.

[ ] QBFT block-header validator selection only.

[ ] Exactly three SP1 programs exist.

[ ] No sequential intermediate-header replay is implemented.

[ ] No validator array is stored in Solidity consensus state.

[ ] Upstream ICS26Router is not forked.

[ ] Membership batching is implemented behind standard ILightClient verifyMembership.

[ ] MAX_BATCH_MESSAGES is exactly 32 for v1.1.

[ ] Production Proof API changes are implemented or explicitly reported as EXTERNAL_CHANGE_REQUIRED.

26.2 QBFT update proof

[ ] All v1.0.1 QBFT update-proof acceptance items remain satisfied.

[ ] One target H can be generated independently of membership proving.

[ ] ConsensusState[H] remains reusable by multiple packet/ack operations.

26.3 EVM MPT

[ ] All v1.0.1 canonical RLP/MPT inclusion and exclusion requirements remain satisfied.

[ ] Membership verifies the Router account proof exactly once per batch.

[ ] Every membership entry verifies its exact storage proof under the shared authenticated Router storageRoot.

[ ] Present(0) is not treated as Absent.

26.4 IBC state binding

[ ] Packet path uses 0x01, receipt 0x02, acknowledgement 0x03.

[ ] Sequence is U64 big-endian.

[ ] Router-facing path contains exactly one raw terminal element.

[ ] Every batch entry derives its logical Router slot inside the guest.

[ ] Packet/ACK expected commitment is bound exactly without re-hashing by the Besu ZK code.

26.5 Batched membership

[ ] Membership accepts 1..32 entries and rejects 0/33.

[ ] All entries share one proofHeight/stateRoot/sourceRouter.

[ ] Duplicate pathDigest entries are rejected.

[ ] Entries are strictly canonical ordered by pathDigest.

[ ] Batch sizes 1, 5, 10, 20, 32 pass native/SP1 execution tests.

[ ] Guest computes the exact Section 6.8 positional batchRoot.

[ ] Membership public values use schemaVersion 2 and contain batchRoot + entryCount.

[ ] FULL verifyMembership binds H/root/Router/root/count and verifies SP1 under MEMBERSHIP_VKEY.

[ ] Successful FULL verification caches batchId.

[ ] CACHED verification requires an already verified batchId.

[ ] Every verifyMembership call binds the exact Router-supplied path/value by a per-item Merkle branch.

[ ] One N-item batch requires exactly one successful SP1 verifier call when submitted FULL then CACHED.

26.6 Non-membership

[ ] All v1.0.1 non-membership and same-height timeout guarantees remain satisfied.

[ ] Non-membership remains a separate program/VKey and is not accidentally accepted through batch membership cache logic.

26.7 Proof API / scheduler

[ ] Compatible proof intents are deduplicated and queued before proving.

[ ] preferredBatchSize defaults to 10.

[ ] maxBatchSize is capped at 32.

[ ] maxBatchWaitMs defaults to 3000 and is configurable.

[ ] One common H is frozen per batch.

[ ] Multi-key eth_getProof is used or an equivalent one-account/N-storage witness is constructed.

[ ] Update jobs for the same route/H are deduplicated.

[ ] Existing ConsensusState[H] suppresses unnecessary update proving.

[ ] Update and membership proving may run concurrently without weakening on-chain ordering/binding.

[ ] Queue/backpressure prevents one active SP1 job per incoming relayer event.

26.8 Solidity/SP1 integration

[ ] ISP1Verifier.verifyProof is used through the pinned verifier route.

[ ] Three VKeys are distinct and mapped correctly.

[ ] Membership schema-v2 public values decode strictly.

[ ] FULL/CACHED membership envelopes decode strictly.

[ ] Batch cache cannot be populated by an invalid SP1 proof.

[ ] Cross-program proof replay fails.

[ ] Conflicting consensus states cannot overwrite history.

[ ] misbehaviour(bytes) still fails closed.

26.9 Tests

[ ] Existing QBFT/MPT/IBC path tests remain green.

[ ] Batch-Merkle Rust/Solidity golden vectors match.

[ ] Membership batch mutation tests cover one bad entry among valid entries.

[ ] Scheduler/witness-builder tests cover dedupe, batching, frozen H, max wait, backpressure, and multi-key proof parsing.

[ ] Real SP1 membership proofs pass for at least batch sizes 1 and 10.

[ ] Release benchmark executes membership sizes 1, 5, 10, 20, 32.

[ ] Solidity real-verifier test proves one FULL plus at least one CACHED item.

[ ] Two-Besu E2E proves batched packet membership and batched ACK membership.

[ ] Negative E2E rejects invalid cached batch/item branches.

26.10 Build/release and deployment

[ ] Clean rebuild reproduces all three ELF hashes.

[ ] Clean setup reproduces all three program VKeys.

[ ] Membership VKey corresponds to schema-v2 batched membership guest.

[ ] Release manifest records MAX_BATCH_MESSAGES and batch-Merkle constants.

[ ] Updated light-client bytecode/ABI hashes are recorded.

[ ] Deployment/migration instructions explain that v1.0.1 cannot be upgraded by changing only a VKey when immutables/bytecode are fixed.

[ ] No secrets/private deployment data exist in repository/release artifacts.

[ ] Performance baseline and batch benchmark matrix are recorded.

27. Normative source references and precedence

The implementation MUST use the pinned source revisions in the release manifest. The paths below identify the upstream behaviors that this specification intentionally mirrors.

27.1 Hyperledger Besu QBFT

Relevant source repositories:

https://github.com/besu-eth/besu
https://github.com/cosmos/ibc-contracts
https://github.com/succinctlabs/sp1
https://github.com/succinctlabs/sp1-contracts

Relevant Besu source areas:

besu/
  consensus/common/src/main/java/org/hyperledger/besu/consensus/common/bft/
    BftHelpers.java
    BftBlockHashing.java

  consensus/common/src/main/java/org/hyperledger/besu/consensus/common/bft/headervalidationrules/
    BftCommitSealsValidationRule.java
    BftValidatorsValidationRule.java

  consensus/qbft/src/main/java/org/hyperledger/besu/consensus/qbft/
    QbftExtraDataCodec.java

Behaviors mirrored:

- target quorum uses ceiling(2*N/3);
- duplicate committers are invalid;
- committers must be validators;
- QBFT extraData contains vanity, validators, vote, round, and commit seals in header-validator mode;
- committed-seal signing hash serializes the header with an empty commit-seal list while retaining the QBFT round.

Besu 26.5.0 (release commit prefix 9b674a5) remains the v1.1 reference/conformance baseline. A deployment manifest may pin another Besu release only if all QBFT/header/RPC compatibility test data passes unchanged. If that release differs materially from these rules, implementation MUST stop with BLOCKED\_ON\_SPEC until this specification is revised.

27.2 Trusted-set light-client model

The skip-update security model is the trusted-validator light-client pattern used by YUI's Besu/IBFT-family light-client work: one target header is checked against both the previously trusted validator authority and the target validator authority. This project uses the explicit thresholds in Section 8 and authenticates the trusted validator array through an on-chain validatorSetHash instead of storing the whole array.
This is intentionally not a replay of Besu's native validator-vote history.

27.3 IBC Solidity contracts

Pinned v1.1 baseline:

repository: cosmos/ibc-contracts
release:    solidity-v3.0.2
commit:     39904319467b5fbdec8c34c43b66af6f1d256dba

The executable upstream Solidity at that commit is authoritative for:

ILightClient ABI
ICS26Router call behavior
ICS24 path construction
packet/acknowledgement commitments
receipt semantics
IBCStore storage namespace/layout
Router-facing path construction supplied to the light client

The repository MUST cross-test these semantics rather than re-document and independently evolve them.

27.4 SP1 and on-chain verifier

Pinned v1.1 baseline:

SP1:          v6.1.0
sp1-contracts: v6.1.1

The integration uses the official verifier interface:

function verifyProof(
    bytes32 programVKey,
    bytes calldata publicValues,
    bytes calldata proofBytes
) external view;

When SP1VerifierGateway is used, its proof selector routing is part of the pinned verifier deployment behavior. This repository MUST not reimplement the proof verifier cryptography.

27.5 Precedence rule

When implementing v1.1, precedence is:

1. This specification for the chosen light-client security model,
   schemas, bindings, repository layout, and test requirements.

2. Exact pinned upstream executable source for protocol encodings,
   Besu committed-seal mechanics, IBC contract semantics, and SP1 verifier ABI.

3. Golden test vectors generated from those pinned sources.

4. Non-normative README/docs/comments.

If (1) and (2) conflict on a behavior that affects proof validity or on-chain acceptance, do not choose one silently. Report BLOCKED\_ON\_SPEC, cite the conflicting source path/version, and revise this specification before changing implementation.

28. Final v1.1 guarantee

A conformant implementation establishes the following statement.

QBFT consensus update

Starting from an on-chain trusted validator-set commitment at height T, the update-client SP1 program authenticates the private full trusted set against that commitment, verifies the exact committed-seal digest and signatures on one target QBFT header H, requires sufficient distinct trusted-validator overlap and the exact Besu target-validator quorum, and commits the target block's height, EVM stateRoot, timestamp, and next validator-set commitment. The Solidity light client binds the proof to its stored trusted state before storing the new consensus state.

Batched membership

Starting from one proof height H and candidate stateRoot R_H, the membership SP1 program verifies the configured Router account once, obtains one authenticated Router storageRoot, and verifies 1..32 exact packet/acknowledgement storage entries under that same storageRoot. It independently derives every Router slot from the exact IBC path, requires every authenticated storage word to equal the expected commitment, canonically orders entries, rejects duplicates, and commits the complete validated set into one positional batchRoot.

The Solidity light client accepts that SP1 batch only when H/root/Router/count/root match a stored ConsensusState[H] and MEMBERSHIP_VKEY. It caches the resulting batchId. Every standard ICS26Router verifyMembership call must still prove that its exact Router-supplied path/value leaf belongs to the authenticated batchRoot. Therefore one expensive SP1 membership proof may safely serve many standard packet or acknowledgement operations without changing ICS26Router.

Update reuse and scheduling

Multiple membership entries and batches may reuse one accepted ConsensusState[H]. If H is not stored, update-client and membership proving may execute concurrently off-chain, but updateClient(H) must establish the consensus state before verifyMembership(H) succeeds on-chain. The Proof API may batch, queue, deduplicate, select H, and apply backpressure; none of those decisions is trusted for correctness.

Non-membership

The non-membership program remains the v1.0.1 single-receipt exclusion proof. It derives the exact receipt slot and proves cryptographic absence under the authenticated Router storage root. Timeout logic uses the authenticated timestamp from the same ConsensusState[H].

Trust boundary

RPC nodes, relayers, the Proof API, scheduler, prover workers, and all witness data may be malicious. Correctness comes from the deterministic SP1 guest checks, immutable program VKeys, the official SP1 verifier, stored consensus state, exact batch-root construction, and the light client's per-item comparison with the IBC Router's operation-derived path and value.

Throughput boundary

This specification changes proof granularity, not IBC correctness semantics:

N packet/ACK membership operations
    -> one shared proof height H
    -> zero or one update proof
    -> one batched membership SP1 proof
    -> N standard Router operations

MAX_BATCH_MESSAGES = 32 is an implementation safety/resource cap. preferredBatchSize = 10 is a runtime starting point that MUST be tuned from actual 1/5/10/20/32 benchmarks.
