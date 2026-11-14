<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Solana IBC Integration Tests

Solana-to-Solana IBC integration tests using `ProgramTest` (BanksClient).

Each chain is an isolated Solana runtime (`ProgramTest` → `BanksClient`) with the same programs deployed independently under identical program IDs. There is no real network between them — the relayer bridges state in-process by reading commitments from one `BanksClient` and submitting delivery transactions to the other. A mock light client accepts any proof, so tests focus on IBC state machine correctness rather than proof verification.

## Architecture

```mermaid
graph LR
    subgraph "Test Harness"
        Test["Test Function"]
        Deployer["Deployer (Actor)"]
        Admin["Admin (Actor)"]
        IftAdmin["IftAdmin (Actor)"]
        User["User (Actor)"]
        Relayer["Relayer (Actor)"]
    end

    subgraph "Chain A (ProgramTest)"
        RA["ics26_router"]
        MCA["mock_light_client"]
        AMA["access_manager"]
        AppA["test_ibc_app / mock_ibc_app / ics27_gmp"]
    end

    subgraph "Chain B (ProgramTest)"
        RB["ics26_router"]
        MCB["mock_light_client"]
        AMB["access_manager"]
        AppB["test_ibc_app / mock_ibc_app / ics27_gmp"]
    end

    Test --> Deployer
    Test --> Admin
    Test --> IftAdmin
    Test --> User
    Test --> Relayer
    Deployer -->|init_ibc_stack + transfer_upgrade_authority| RA
    Deployer -->|init_ibc_stack + transfer_upgrade_authority| RB
    User -->|send_packet / send_call| RA
    Admin -->|AM transfer| RA
    Relayer -->|upload_chunks + recv_packet| RB
    Relayer -->|upload_chunks + ack_packet| RA
    Relayer -->|upload_chunks + timeout_packet| RA
    RA -->|CPI| MCA
    RA -->|CPI| AMA
    RA -->|CPI| AppA
    RB -->|CPI| MCB
    RB -->|CPI| AMB
    RB -->|CPI| AppB
```

## Three-Phase Chain Lifecycle

Each `Chain` follows a setup → init → runtime lifecycle:

```mermaid
flowchart LR
    subgraph Setup["Setup Phase"]
        direction TB
        New["Chain::single / pair / pair_with / new"] --> Prefund["prefund(&[actors])"]
    end

    subgraph Init["Init Phase"]
        direction TB
        ChainInit["chain.init(deployer, admin, relayer, programs)\n(= start + init_ibc_stack + transfer_upgrade_authority)"]
        ChainInit -.->|IFT etc.| InitApp["deployer.init_programs()"]
    end

    subgraph Runtime["Runtime Phase"]
        direction TB
        Send["User: send_packet / send_call"] --> Upload["Relayer: upload_chunks"]
        Upload --> Deliver["Relayer: recv_packet / ack_packet / timeout_packet"]
        Deliver --> Verify["get_account + assertions"]
    end

    Setup --> Init --> Runtime
```

**Setup phase** — `Chain::single(deployer, programs)`, `Chain::pair(deployer, programs)`, `Chain::pair_with(deployer, programs_a, programs_b)` or the lower-level `Chain::new(ChainConfig { .. })` configure `ProgramTest` with program binaries and `ProgramData` accounts (for upgrade authority verification). `single` / `pair` default to the `chain-a-client` ↔ `chain-b-client` IDs; use `new` when a test needs custom client IDs (e.g. the three-chain test). Only the deployer is pre-funded automatically; other actors must be pre-funded explicitly via `chain.prefund(&[&admin, &relayer, &user])`. Per-PDA lamport top-ups (e.g. for GMP account PDAs) go through `chain.prefund_lamports(pda, GMP_ACCOUNT_PREFUND_LAMPORTS)` and must happen before `start()`. No on-chain state exists yet.

**Init phase** — `chain.init(deployer, admin, relayer, programs)` is the canonical one-shot helper that wraps `start()` + `deployer.init_ibc_stack()` + `deployer.transfer_upgrade_authority()`. Tests that need to interleave extra steps (e.g. `deployer.init_programs()` for IFT, or `deployer.add_counterparty()` for multi-hop) drop down to the individual calls. Inside `init_ibc_stack` the deployer signs upgrade-authority-gated steps while the admin signs AM-role-gated steps:

1. `access_manager::initialize` — creates the AM account with admin's pubkey as `ADMIN_ROLE` holder *(deployer signs)*
2. `access_manager::grant_role` — grants `RELAYER_ROLE` to relayer and `ID_CUSTOMIZER_ROLE` to admin *(admin signs)*
3. `ics26_router::initialize` — creates the router state *(deployer signs)*
4. `mock_light_client::initialize` — creates client and consensus state accounts
5. `add_client` + `add_ibc_app` — registers the light client and IBC application *(admin signs)*
6. App-specific initialization (`test_ibc_app::initialize`, `ics27_gmp::initialize` + `test_gmp_app::initialize`, or nothing for `mock_ibc_app`) *(deployer signs)*

Programs that need a different admin (e.g. IFT) are initialized separately via `deployer.init_programs(chain, ift_admin_pubkey, &[&Ift])`, which runs only the program-specific `init_steps` with the given admin pubkey.

Finally, `deployer.transfer_upgrade_authority()` transfers upgrade authority of all programs to the access manager PDA so governance controls upgrades *(deployer signs)*. `Chain::init` runs this step automatically.

**Runtime phase** — actors submit transactions and read account state.

## Program Variants

Each struct in `programs.rs` implements the `ChainProgram` trait. IBC application variants register on a port and run initialization; auxiliary variants only load the binary.

| Variant        | Program loaded   | Port registration | Behavior                                                          |
| -------------- | ---------------- | ----------------- | ----------------------------------------------------------------- |
| `TestIbcApp`   | `test_ibc_app`   | Yes               | Stateful app that counts packets sent/received/acked/timed-out    |
| `MockIbcApp`   | `mock_ibc_app`   | Yes               | Stateless app with magic-string ack control (`RETURN_ERROR_ACK` etc.) |
| `Ics27Gmp`     | `ics27_gmp`      | Yes               | GMP IBC application on the GMP port                               |
| `TestGmpApp`   | `test_gmp_app`   | No                | Counter app invoked by GMP via CPI                                |
| `TestCpiProxy` | `test_cpi_proxy` | No                | Generic CPI proxy for security tests                              |
| `Ift`          | `ift`            | No                | Inter-chain fungible token transfers (uses GMP's port)            |
| `TestAccessManager` | `test_access_manager` | No           | Second AM instance for access manager migration tests             |

## Module Overview

| Module     | Purpose                                                                                              |
| ---------- | ---------------------------------------------------------------------------------------------------- |
| `chain`    | `Chain` struct with setup/runtime lifecycle, `ChainConfig`, `ChainProgram` trait and PDA derivation helpers |
| `programs` | `ChainProgram` implementations: `TestIbcApp`, `MockIbcApp`, `Ics27Gmp`, `TestGmpApp`, `TestCpiProxy`, `Ift`, `TestAccessManager` |
| `accounts` | `anchor_discriminator` and `account_owned_by` helpers                                                |
| `actors`   | `Actor` trait and actor modules (`deployer`, `admin`, `ift_admin`, `user`, `relayer`)                 |
| `router`   | Instruction builders for `send_packet`, `recv_packet`, `ack_packet`, `timeout_packet`, chunk uploads, AM transfer (propose/accept/cancel) and `read_router_state` |
| `gmp`      | Instruction builders for GMP `send_call`, `recv_packet`, `ack_packet`, `timeout_packet`, raw `on_recv_packet` for security tests, AM transfer (propose/accept/cancel) and `read_gmp_app_state` |
| `ift`      | Instruction builders for IFT transfers, finalization, admin operations, pause, token creation (SPL and Token 2022), `TokenKind` enum and balance readers |

## Actors

```mermaid
graph TB
    Actor["trait Actor\npubkey()"]
    Deployer["Deployer\n- upgrade authority holder\n- init_ibc_stack\n- init_programs\n- transfer_upgrade_authority\n- add_counterparty"]
    Admin["Admin\n- ics26_propose/accept/cancel_am_transfer\n- gmp_propose/accept/cancel_am_transfer"]
    IftAdmin["IftAdmin\n- set_paused\n- propose/accept/cancel_admin\n- admin_mint"]
    User["User\n- send_packet\n- send_call\n- ift_transfer"]
    Relayer["Relayer\n- upload_chunks\n- upload_chunks_for_client\n- upload_chunks_with_multi_proof\n- cleanup_chunks\n- recv_packet\n- recv_packet_multi_proof\n- ack_packet\n- ack_packet_multi_proof\n- timeout_packet\n- gmp_recv_packet\n- gmp_ack_packet\n- gmp_timeout_packet\n- ift_gmp_ack_packet\n- ift_gmp_timeout_packet\n- ift_finalize_transfer"]

    Actor --> Deployer
    Actor --> Admin
    Actor --> IftAdmin
    Actor --> User
    Actor --> Relayer
```

All actors wrap a `Keypair`. `Deployer` holds the upgrade authority and orchestrates program initialization via `init_ibc_stack()`, then transfers upgrade authority to the access manager PDA via `transfer_upgrade_authority()`. Programs that need a different admin than the core stack (e.g. IFT) are initialized separately via `init_programs()`. For multi-hop tests, `add_counterparty()` registers additional client/counterparty pairs. `Admin` is an independent keypair whose pubkey is passed to the AM `initialize` instruction as the admin — it manages AM operations (role grants, AM transfers for ICS26 Router and GMP). `IftAdmin` manages IFT-specific admin operations (pause, admin transfer, minting) — a separate concern from the AM admin. `User` initiates IBC sends; `Relayer` bridges packets between chains and holds the `RELAYER_ROLE` in the access manager.

## Packet Flow

Before each packet delivery, the relayer uploads payload and proof data to on-chain chunk PDAs via `upload_payload_chunk`/`upload_proof_chunk` transactions. The router reads those chunks during instruction execution.

### Router: send → recv → ack

```mermaid
graph LR
    U["User"] -->|send_packet| A["Chain A\n(commitment created)"]
    A -->|"relayer observes commitment"| R["Relayer"]
    R -->|"upload_chunks + recv_packet"| B["Chain B\n(receipt + ack created)"]
    B -->|"relayer observes ack"| R
    R -->|"upload_chunks + ack_packet"| A2["Chain A\n(commitment zeroed)"]
```

### Router: send → timeout

```mermaid
graph LR
    U["User"] -->|send_packet| A["Chain A\n(commitment created)"]
    A -->|"packet expires"| R["Relayer"]
    R -->|"upload_chunks + timeout_packet"| A2["Chain A\n(commitment zeroed)"]
```

### GMP: send_call → recv → ack

```mermaid
graph LR
    U["User"] -->|send_call| A["Chain A\n(commitment created)"]
    A -->|"relayer observes commitment"| R["Relayer"]
    R -->|"upload_chunks + gmp_recv_packet"| B["Chain B\n(receipt + ack + app CPI)"]
    B -->|"relayer observes ack"| R
    R -->|"upload_chunks + gmp_ack_packet"| A2["Chain A\n(commitment zeroed\n+ GMPCallResult)"]
```

### GMP: send_call → timeout

```mermaid
graph LR
    U["User"] -->|send_call| A["Chain A\n(commitment created)"]
    A -->|"packet expires"| R["Relayer"]
    R -->|"upload_chunks + gmp_timeout_packet"| A2["Chain A\n(commitment zeroed\n+ GMPCallResult timeout)"]
```

### IFT: transfer → ack → finalize

```mermaid
graph LR
    U["User"] -->|ift_transfer| A["Chain A\n(tokens burned,\ncommitment + PendingTransfer)"]
    A -->|"relayer observes commitment"| R["Relayer"]
    R -->|"upload_chunks + ift_gmp_ack_packet"| A2["Chain A\n(commitment zeroed\n+ GMPCallResult)"]
    R -->|"finalize_transfer"| A3["Chain A\n(PendingTransfer closed,\nsuccess: no-op / error: refund)"]
```

The IFT module supports both SPL Token and Token 2022 mints via the `TokenKind` enum. Tests use `setup_ift_chain` (SPL) or `setup_ift_chain_with_token` (either variant) to create a token, register an EVM bridge and mint an initial balance.

#### IFT Test Coverage

| Test | Scenario |
| --- | --- |
| `full_lifecycle` | Transfer → success ack → finalize (tokens stay burned) |
| `error_ack_refund` | Transfer → error ack → finalize (tokens refunded) |
| `timeout_refund` | Transfer → timeout → finalize (tokens refunded) |
| `batch_transfers` | Two consecutive transfers (seq 1 & 2), both acked and finalized |
| `token_2022_lifecycle` | Full lifecycle with Token 2022 mint (metadata extensions) |
| `admin_transfer` | Propose → accept admin; propose → cancel admin |
| `pause` | Pause blocks transfer + admin_mint; unpause restores them |

#### Admin Test Coverage

| Test | Scenario |
| --- | --- |
| `ics26_am_transfer_propose_accept` | Propose AM transfer on ICS26 Router, accept, verify `RouterState.am_state` updated |
| `ics26_am_transfer_propose_cancel` | Propose, cancel, verify pending cleared and AM unchanged |
| `ics26_am_transfer_unauthorized_propose` | Non-admin propose fails with `Unauthorized` |
| `gmp_am_transfer_propose_accept` | Propose AM transfer on GMP, accept, verify `GMPAppState.am_state` updated |
| `gmp_am_transfer_propose_cancel` | Propose, cancel on GMP |
| `gmp_am_transfer_unauthorized_propose` | Non-admin propose fails on GMP |

## Writing a New Test

Tests are organized into labelled sections so the setup → runtime → assertions flow is consistent across the suite. The canonical layout is:

```text
// ── Actors ──         actor declarations only (deployer, admin, user, relayer, ...)
// ── Test data ──      sequences, amounts, packet_data, acks, ... (plain values only)
// ── Chains ──         programs slice, Chain::single / pair + prefund + prefund_lamports
// ── Init ──           chain.init(...) (or start / init_ibc_stack / init_programs / add_counterparty / transfer_upgrade_authority)
// ── Build payload ──  PDA derivations, payload encoding (after init so on-chain state is available)
// ── <scenario> ──     send → recv → ack / timeout, assertions
```

The `programs` slice lives in the `// ── Chains ──` section (next to the `Chain::pair` call that consumes it), not in `// ── Test data ──`. Single-chain tests can drop the `// ── Test data ──` section entirely if there is no plain test data to declare.

A minimal router test that sends a packet from Chain A, delivers it to Chain B and acknowledges it back:

```rust
use super::*;

#[tokio::test]
async fn test_my_scenario() {
    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let sequence = 1u64;
    let packet_data = b"hello from chain A";
    let successful_ack = br#"{"result": "AQ=="}"#.to_vec();

    // ── Chains ──
    let programs: &[&dyn ChainProgram] = &[&TestIbcApp];
    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer]);

    // ── Init ──
    chain_a.init(&deployer, &admin, &relayer, programs).await;
    chain_b.init(&deployer, &admin, &relayer, programs).await;

    // ── Send on Chain A ──
    let send = user
        .send_packet(
            &mut chain_a,
            SendPacketParams {
                sequence,
                packet_data,
            },
        )
        .await
        .expect("send failed");

    assert_commitment_set(&chain_a, send.commitment_pda).await;

    // ── Recv on Chain B ──
    let (payload_pda, proof_pda) = relayer
        .upload_chunks(&mut chain_b, sequence, packet_data, DUMMY_PROOF)
        .await
        .expect("upload failed");

    let recv = relayer
        .recv_packet(
            &mut chain_b,
            RecvPacketParams {
                sequence,
                payload_chunk_pda: payload_pda,
                proof_chunk_pda: proof_pda,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("recv failed");

    assert_receipt_created(&chain_b, recv.receipt_pda).await;

    // ── Ack on Chain A ──
    let (ack_payload, ack_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, DUMMY_PROOF)
        .await
        .expect("upload failed");

    let commitment_pda = relayer
        .ack_packet(
            &mut chain_a,
            AckPacketParams {
                sequence,
                acknowledgement: successful_ack,
                payload_chunk_pda: ack_payload,
                proof_chunk_pda: ack_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("ack failed");

    assert_commitment_zeroed(&chain_a, commitment_pda).await;
}
```

Key patterns:

- **Section markers** — group code into `// ── Actors ──`, `// ── Test data ──`, `// ── Chains ──`, `// ── Init ──`, `// ── Build payload ──` and per-scenario sections. Keep actor declarations separate from test data so the intent of each binding is obvious at a glance.
- **Chain constructors** — prefer `Chain::pair` (two chains, shared programs), `Chain::pair_with` (two chains, different programs) and `Chain::single` (one chain). Fall back to `Chain::new(ChainConfig { .. })` only when a test needs custom client IDs (e.g. `three_chain`).
- **Prefund before init** — `chain.prefund(&[actors])` and `chain.prefund_lamports(pda, GMP_ACCOUNT_PREFUND_LAMPORTS)` must happen before `chain.init(...)` because `ProgramTest::start` consumes the builder.
- **One-shot init** — `chain.init(&deployer, &admin, &relayer, programs).await` wraps `start` + `init_ibc_stack` + `transfer_upgrade_authority`. Programs with a separate admin (like IFT) are excluded from `init_ibc_stack` and initialized via `deployer.init_programs(chain, ift_admin.pubkey(), &[&Ift])` between `start` and `transfer_upgrade_authority`. For multi-hop, call `deployer.add_counterparty()` before `transfer_upgrade_authority`.
- **Shared constants** — use `DUMMY_PROOF` for the 32-byte mock proof, `GMP_ACCOUNT_PREFUND_LAMPORTS` / `GMP_PAYLOAD_PREFUND_LAMPORTS` for GMP prefund amounts, `GMP_TIMEOUT` / `IFT_TIMEOUT` for packet timeouts. Don't inline these literals.
- **Chunks before delivery** — every `recv_packet`, `ack_packet` and `timeout_packet` requires a preceding `upload_chunks` call.
- **`Default::default()`** — param structs implement `Default` for fields like `timeout_timestamp` and `proof_height`, so you only need to set what matters for your test.
- **Error assertions** — use `extract_custom_error` to match specific Anchor error codes instead of just checking that a transaction failed.

## Helper Functions

| Function | What it does | When to use |
| --- | --- | --- |
| `assert_commitment_set(chain, pda)` | Checks the commitment PDA has non-zero data | After `send_packet` to verify the commitment was stored |
| `assert_commitment_zeroed(chain, pda)` | Checks the commitment PDA was zeroed out | After `ack_packet` or `timeout_packet` to confirm consumption |
| `assert_receipt_created(chain, pda)` | Checks the receipt PDA exists and is owned by the router | After `recv_packet` to verify replay protection |
| `extract_ack_data(chain, pda)` | Reads the 32-byte ack commitment from a PDA | When you need to inspect the acknowledgement content |
| `extract_custom_error(err)` | Extracts the `u32` error code from a `BanksClientError` | When asserting a transaction failed with a specific Anchor error |
| `anchor_error_code(discriminant)` | Computes `6000 + discriminant` for an Anchor error variant | When constructing expected error codes from enum variants |

Pre-computed error constants are also available: `PACKET_COMMITMENT_MISMATCH`, `ASYNC_ACK_NOT_SUPPORTED`.

## Shared Constants

| Constant | Defined in | Meaning |
| --- | --- | --- |
| `DUMMY_PROOF` | `integration_tests` (`src/lib.rs`) | 32-byte zero proof accepted by the mock light client — pass as the `&[u8]` proof argument to `upload_chunks` / `upload_chunks_for_client` |
| `GMP_ACCOUNT_PREFUND_LAMPORTS` | `tests/gmp/main.rs` | Lamports to top up a `GMPAccount` PDA so the counter program can pay rent for the user-counter PDA it creates on receipt |
| `GMP_PAYLOAD_PREFUND_LAMPORTS` | `tests/gmp/main.rs` | Lamports requested via `RawGmpSolanaPayload.prefund_lamports` to fund the payload PDA before executing the wrapped instruction |
| `GMP_TIMEOUT` / `GMP_TIMEOUT_TOO_LONG` | `tests/gmp/main.rs` | GMP packet timeout at/within `MAX_TIMEOUT_DURATION`, and one just past it (used to exercise the `TimeoutTooLong` rejection) |
| `IFT_TIMEOUT` | `tests/ift/main.rs` | IFT packet timeout aligned with `router::test_timeout(TEST_CLOCK_TIME)` |
| `MINT_DECIMALS` / `INITIAL_BALANCE` / `TRANSFER_AMOUNT` | `tests/ift/main.rs` | IFT token setup and transfer amounts |

## Running

```bash
# Run integration tests after building the required program binaries
just solana::test-solana-integration

# Or run all integration tests directly (after builds)
cd ibc-solana && cargo test -p integration-tests

# Run a specific test with logs
cd ibc-solana && cargo test -p integration-tests test_full_packet_lifecycle -- --nocapture
```

Programs are loaded from `target/deploy/` via `SBF_OUT_DIR`. After modifying any program source, rebuild with `just solana::build-solana-programs <program>` before re-running tests.
