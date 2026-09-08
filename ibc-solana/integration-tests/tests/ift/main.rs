// SPDX-License-Identifier: Apache-2.0

//! Solana IFT (Inter-chain Fungible Token) integration tests.
//!
//! Each test spins up one or more `ProgramTest` chains with IFT + GMP and the
//! attestation light client. Most tests use a single chain that burns tokens
//! locally and dispatches a GMP call to a simulated EVM counterparty;
//! `solana_roundtrip` exercises a real Solana-to-Solana transfer across a pair.
//!
//! ## Coverage gaps (not testable at integration level)
//!
//! - **`initialize_existing_token`**: requires an externally-created mint whose
//!   authority signs the transfer — complex setup but doable later.
//! - **Batch relay (single tx with multiple recv/ack)**: tested in e2e only.

use anchor_spl::token::spl_token::error::TokenError;
use integration_tests::{
    admin::Admin,
    assert_commitment_set, assert_commitment_zeroed, attestation,
    attestor::Attestors,
    chain::{Chain, ChainProgram, TEST_CLOCK_TIME},
    deployer::Deployer,
    extract_ack_data, extract_custom_error,
    gmp::{self, GmpAckPacketParams, GmpRecvPacketParams},
    ift::{self, IftGmpAckPacketParams, IftGmpTimeoutPacketParams, IftTransferParams, TokenKind},
    ift_admin::IftAdmin,
    programs::{AttestationLc, Ics27Gmp, Ift},
    read_commitment,
    relayer::Relayer,
    router::PROOF_HEIGHT,
    user::User,
    Actor,
};
use solana_program_test::BanksClientError;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

/// Assert that a transaction failure is a specific `spl_token::TokenError`.
///
/// Wraps `extract_custom_error` with type-safe comparison against a
/// `TokenError` variant, so tests don't have to hand-encode error discriminants.
fn assert_spl_token_error(err: &BanksClientError, expected: TokenError) {
    let actual = extract_custom_error(err);
    let expected_code = expected.clone() as u32;
    assert_eq!(
        actual, expected_code,
        "expected SPL Token error {expected:?} (code {expected_code}), got code {actual}"
    );
}

/// Initialize a chain with the IFT+GMP stack and attestation LC.
///
/// `admin` controls the IBC router/client/GMP app; `ift_admin` controls the
/// IFT program's own app state. Both sets of programs transfer their upgrade
/// authority to the access manager PDA. An `update_client` at `PROOF_HEIGHT`
/// is submitted so the chain is ready for proof verification.
async fn init_ift_chain(
    chain: &mut Chain,
    deployer: &Deployer,
    admin: &Admin,
    ift_admin: &IftAdmin,
    relayer: &Relayer,
    attestors: &Attestors,
    attestation_lc: &AttestationLc,
) {
    let ibc_programs: &[&dyn ChainProgram] = &[&Ics27Gmp, attestation_lc];
    let all_programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &Ift, attestation_lc];
    chain.start().await;
    deployer
        .init_ibc_stack(chain, admin, relayer, ibc_programs)
        .await;
    deployer
        .init_programs(chain, ift_admin.pubkey(), &[&Ift])
        .await;
    deployer
        .transfer_upgrade_authority(chain, all_programs)
        .await;
    relayer
        .attestation_update_client(chain, attestors, PROOF_HEIGHT)
        .await
        .expect("attestation update_client failed");
}

mod admin_transfer;
mod batch_ack_single_tx;
mod batch_transfers;
mod error_ack_refund;
mod full_lifecycle;
mod pause;
mod solana_roundtrip;
mod three_chain;
mod timeout_refund;
mod token_2022_lifecycle;

/// IFT timeout: must match `router::test_timeout(TEST_CLOCK_TIME)`.
const IFT_TIMEOUT: u64 = TEST_CLOCK_TIME as u64 + 86_000;

const MINT_DECIMALS: u8 = 6;
const INITIAL_BALANCE: u64 = 1_000_000;
const TRANSFER_AMOUNT: u64 = 100_000;

/// Lamports prefunded into the destination-side GMP account PDA so that it
/// has enough balance to pay ATA rent for the `ift_mint` CPI.
/// Mirrors `GMP_ACCOUNT_PREFUND_LAMPORTS` in `tests/gmp/main.rs`.
const GMP_ACCOUNT_PREFUND_LAMPORTS: u64 = 10_000_000;

/// Set up a chain with IFT + GMP, create an SPL token, register a bridge and
/// mint tokens to the user.
///
/// Returns `(mint_pubkey, user_ata)`.
async fn setup_ift_chain(
    chain: &mut Chain,
    ift_admin: &IftAdmin,
    mint_keypair: &Keypair,
    user_pubkey: Pubkey,
) -> (Pubkey, Pubkey) {
    setup_ift_chain_with_token(chain, ift_admin, mint_keypair, user_pubkey, TokenKind::Spl).await
}

/// Set up a chain with IFT + GMP, create a token (SPL or Token 2022), register
/// a bridge and mint tokens to the user.
///
/// Returns `(mint_pubkey, user_ata)`.
async fn setup_ift_chain_with_token(
    chain: &mut Chain,
    ift_admin: &IftAdmin,
    mint_keypair: &Keypair,
    user_pubkey: Pubkey,
    token_kind: TokenKind,
) -> (Pubkey, Pubkey) {
    let mint = mint_keypair.pubkey();
    let admin_pubkey = ift_admin.pubkey();

    // 1. Create token (IFT admin pays + signs as authority)
    let create_token_ix = match token_kind {
        TokenKind::Token2022 => ift::build_create_token_2022_ix(
            admin_pubkey,
            admin_pubkey,
            mint,
            MINT_DECIMALS,
            "Test Token".to_string(),
            "TT".to_string(),
            "https://example.com".to_string(),
        ),
        TokenKind::Spl => {
            ift::build_create_spl_token_ix(admin_pubkey, admin_pubkey, mint, MINT_DECIMALS)
        }
    };
    let tx = Transaction::new_signed_with_payer(
        &[create_token_ix],
        Some(&admin_pubkey),
        &[ift_admin.keypair(), mint_keypair],
        chain.blockhash(),
    );
    chain.process_transaction(tx).await.expect("create token");

    // 2. Register EVM bridge (IFT admin pays + signs)
    let register_bridge_ix = ift::build_register_bridge_ix(
        admin_pubkey,
        admin_pubkey,
        mint,
        chain.client_id(),
        ift::COUNTERPARTY_IFT_ADDRESS,
    );
    let tx = Transaction::new_signed_with_payer(
        &[register_bridge_ix],
        Some(&admin_pubkey),
        &[ift_admin.keypair()],
        chain.blockhash(),
    );
    chain
        .process_transaction(tx)
        .await
        .expect("register bridge");

    // 3. Mint tokens to user's ATA (IFT admin pays + signs)
    let admin_mint_ix = ift::build_admin_mint_ix(
        admin_pubkey,
        admin_pubkey,
        mint,
        user_pubkey,
        INITIAL_BALANCE,
        token_kind,
    );
    let tx = Transaction::new_signed_with_payer(
        &[admin_mint_ix],
        Some(&admin_pubkey),
        &[ift_admin.keypair()],
        chain.blockhash(),
    );
    chain.process_transaction(tx).await.expect("admin mint");

    let user_ata = token_kind.get_ata(&user_pubkey, &mint);
    (mint, user_ata)
}

/// Prove that the one-time `mint_keypair` used during token creation retains
/// no residual authority after initialization.
///
/// `mint_keypair` only signs once — for the SPL Token `initialize_mint`
/// instruction where its pubkey designates the new mint account. After that
/// step the mint's on-chain authority is the program-derived
/// `mint_authority_pda`, so the original keypair becomes a plain address
/// with no powers over the token supply or configuration.
///
/// This helper verifies that property three different ways:
/// 1. Read the mint account and pin `mint_authority` to the program PDA.
/// 2. Attempt `set_authority` signed by `mint_keypair` — must be rejected
///    with `OwnerMismatch` (can't hand the mint over to itself).
/// 3. Attempt `mint_to` signed by `mint_keypair` — must be rejected with
///    `OwnerMismatch` (can't inflate the supply).
///
/// The transactions are paid by `user` because `mint_keypair` has no lamports
/// to cover fees; `mint_keypair` still signs the instruction it is claiming
/// authority for, which is what the SPL Token program actually checks.
async fn assert_mint_keypair_powerless(
    chain: &mut Chain,
    user: &User,
    mint_keypair: &Keypair,
    mint: Pubkey,
) {
    use anchor_spl::token::spl_token;

    // ── 1. Positive: on-chain authority is the program PDA, not mint_keypair ──
    // `create_and_initialize_spl_token` passes the PDA as `initialize_mint`'s
    // mint-authority argument, so the SPL Token program writes the PDA into
    // the mint account. Reading it back and comparing is the strongest form
    // of the check: it names the one and only pubkey that can mint/freeze.
    let mint_state = ift::read_spl_mint(chain, mint).await;
    let mint_authority_pda = ift::derive_mint_authority_pda(&mint);
    assert_eq!(
        Option::<Pubkey>::from(mint_state.mint_authority),
        Some(mint_authority_pda),
        "mint authority must be the program PDA"
    );

    // ── 2. Negative: mint_keypair cannot reassign the mint authority ──
    // Build a raw SPL Token `set_authority` trying to hand the MintTokens
    // authority over to mint_keypair itself, with mint_keypair signing as the
    // "current" authority. The SPL Token program compares the signer against
    // the mint's stored authority (the PDA) and rejects the instruction.
    let hijack_ix = spl_token::instruction::set_authority(
        &spl_token::ID,
        &mint,
        Some(&mint_keypair.pubkey()),
        spl_token::instruction::AuthorityType::MintTokens,
        &mint_keypair.pubkey(),
        &[],
    )
    .expect("build set_authority ix");
    // user pays fees; mint_keypair signs the set_authority instruction itself.
    let tx = Transaction::new_signed_with_payer(
        &[hijack_ix],
        Some(&user.pubkey()),
        &[user.keypair(), mint_keypair],
        chain.blockhash(),
    );
    let err = chain
        .process_transaction(tx)
        .await
        .expect_err("set_authority signed by mint_keypair must fail");
    assert_spl_token_error(&err, TokenError::OwnerMismatch);

    // ── 3. Negative: mint_keypair cannot mint new supply ──
    // Build a raw SPL Token `mint_to` issuing 1 token unit to the user's ATA
    // (which already exists from `setup_ift_chain`) with mint_keypair signing
    // as the claimed authority. Same runtime check, same rejection.
    let user_ata = TokenKind::Spl.get_ata(&user.pubkey(), &mint);
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &user_ata,
        &mint_keypair.pubkey(),
        &[],
        1,
    )
    .expect("build mint_to ix");
    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&user.pubkey()),
        &[user.keypair(), mint_keypair],
        chain.blockhash(),
    );
    let err = chain
        .process_transaction(tx)
        .await
        .expect_err("mint_to signed by mint_keypair must fail");
    assert_spl_token_error(&err, TokenError::OwnerMismatch);
}

/// Create an SPL Token mint under IFT control.
async fn create_spl_token(chain: &mut Chain, ift_admin: &IftAdmin, mint_keypair: &Keypair) {
    let admin_pubkey = ift_admin.pubkey();
    let ix = ift::build_create_spl_token_ix(
        admin_pubkey,
        admin_pubkey,
        mint_keypair.pubkey(),
        MINT_DECIMALS,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin_pubkey),
        &[ift_admin.keypair(), mint_keypair],
        chain.blockhash(),
    );
    chain
        .process_transaction(tx)
        .await
        .expect("create SPL token");
}

/// Register an IFT bridge whose counterparty is another Solana IFT program.
async fn register_solana_bridge(
    chain: &mut Chain,
    ift_admin: &IftAdmin,
    mint: Pubkey,
    client_id: String,
    counterparty_mint: Pubkey,
    counterparty_client_id: String,
) {
    let admin_pubkey = ift_admin.pubkey();
    let ix = ift::build_register_bridge_solana_ix(
        admin_pubkey,
        admin_pubkey,
        mint,
        &client_id,
        ::ift::ID,
        counterparty_mint,
        &counterparty_client_id,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin_pubkey),
        &[ift_admin.keypair()],
        chain.blockhash(),
    );
    chain
        .process_transaction(tx)
        .await
        .expect("register Solana IFT bridge");
}

/// Mint `INITIAL_BALANCE` tokens to the user's ATA via the IFT admin.
async fn admin_mint_to_user(
    chain: &mut Chain,
    ift_admin: &IftAdmin,
    mint: Pubkey,
    user_pubkey: Pubkey,
) {
    let admin_pubkey = ift_admin.pubkey();
    let ix = ift::build_admin_mint_ix(
        admin_pubkey,
        admin_pubkey,
        mint,
        user_pubkey,
        INITIAL_BALANCE,
        TokenKind::Spl,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin_pubkey),
        &[ift_admin.keypair()],
        chain.blockhash(),
    );
    chain.process_transaction(tx).await.expect("admin mint");
}
