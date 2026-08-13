// SPDX-License-Identifier: Apache-2.0

use access_manager::AccessManagerState;
use anchor_lang::{
    AccountSerialize, AnchorDeserialize, AnchorSerialize, Discriminator, InstructionData,
    ToAccountMetas,
};
use ed25519_dalek::{Signer, SigningKey};
use ics07_tendermint::state::SignatureVerification;
use rstest::{fixture, rstest};
use sha2::{Digest, Sha256};
use solana_ibc_types::ics07::SignatureData;
use solana_program_test::{BanksClient, ProgramTest, ProgramTestBanksClientExt};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, hash::Hash, instruction::Instruction, pubkey::Pubkey,
    signature::Keypair, signer::Signer as SolSigner, sysvar::instructions as ix_sysvar,
    transaction::Transaction,
};

const PROGRAM_BINARY_PATH: &str = "../../target/deploy/ics07_tendermint";

/// Creates a `ProgramTest` instance with the `ics07_tendermint` program loaded
/// and pre-creates `AppState` and `AccessManager` PDAs so that the payer has `RELAYER_ROLE`.
fn setup_program_test() -> ProgramTest {
    if std::env::var("SBF_OUT_DIR").is_err() {
        let deploy_dir = std::path::Path::new(PROGRAM_BINARY_PATH)
            .parent()
            .expect("Invalid program path");
        std::env::set_var("SBF_OUT_DIR", deploy_dir);
    }
    let mut pt = ProgramTest::new("ics07_tendermint", ics07_tendermint::ID, None);
    pt.add_program("access_manager", access_manager::ID, None);

    // Pre-create AppState PDA pointing to access_manager program
    let (app_state_pda, _) =
        Pubkey::find_program_address(&[ics07_tendermint::AppState::SEED], &ics07_tendermint::ID);
    let app_state = ics07_tendermint::AppState {
        am_state: AccessManagerState::new(access_manager::ID),
        _reserved: [0; 256],
    };
    let mut app_data = Vec::new();
    app_state.try_serialize(&mut app_data).unwrap();
    pt.add_account(
        app_state_pda,
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: app_data,
            owner: ics07_tendermint::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Pre-create AccessManager PDA with RELAYER_ROLE for default payer
    // The payer pubkey is deterministic in ProgramTest (first keypair), so we grant
    // the role to all callers by using a broad approach: we'll re-create per test.
    // For simplicity, we skip pre-creating the AccessManager here and do it in the fixture.
    pt
}

fn add_access_manager_with_relayer(pt: &mut ProgramTest, relayer: &Pubkey) {
    let (access_manager_pda, _) = Pubkey::find_program_address(
        &[access_manager::state::AccessManager::SEED],
        &access_manager::ID,
    );
    let am = access_manager::state::AccessManager {
        roles: vec![
            access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![*relayer],
            },
            access_manager::RoleData {
                role_id: solana_ibc_types::roles::RELAYER_ROLE,
                members: vec![*relayer],
            },
        ],
        whitelisted_programs: vec![],
        pending_authority_transfers: vec![],
    };
    let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
    am.serialize(&mut am_data).unwrap();
    pt.add_account(
        access_manager_pda,
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: am_data,
            owner: access_manager::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
}

struct TestContext {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
}

#[fixture]
async fn ctx() -> TestContext {
    let mut pt = setup_program_test();
    // We need to start the test to get the payer, but we also need the payer's pubkey
    // to set up the access manager. ProgramTest payer is deterministic but not easily
    // accessible before start. Instead, we'll use a known keypair as the submitter.
    // Actually, the simplest approach: start without access manager, then we know
    // the payer won't have the role. But existing tests need the payer to have the role.
    //
    // Alternative: pre-create access manager granting role to a "well-known" keypair,
    // and use that as the payer. But ProgramTest generates its own payer.
    //
    // Simplest fix: grant RELAYER_ROLE to ALL pubkeys by using a large member list.
    // No, that's bad. Instead, we accept that the payer pubkey is random and grant
    // role broadly. Actually ProgramTest pays generate a random payer.
    //
    // The actual fix: since ProgramTest payer is random, we create a separate relayer
    // keypair, fund it, and use it as the submitter (not the fee payer).
    let relayer = Keypair::new();
    add_access_manager_with_relayer(&mut pt, &relayer.pubkey());
    pt.add_account(
        relayer.pubkey(),
        solana_sdk::account::Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    let (banks_client, payer, recent_blockhash) = pt.start().await;
    // We store the relayer as "payer" in the context since tests use it as submitter
    let _ = payer; // fee payer not needed directly
    TestContext {
        banks_client,
        payer: relayer,
        recent_blockhash,
    }
}

/// Creates a valid `SignatureData` struct from a signing key and message.
fn create_signature_data(signing_key: &SigningKey, msg: &[u8]) -> SignatureData {
    let pubkey: [u8; 32] = signing_key.verifying_key().to_bytes();
    let signature: [u8; 64] = signing_key.sign(msg).to_bytes();
    create_signature_data_raw(pubkey, msg, signature)
}

fn create_signature_data_raw(pubkey: [u8; 32], msg: &[u8], signature: [u8; 64]) -> SignatureData {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    hasher.update(msg);
    hasher.update(signature);
    let signature_hash: [u8; 32] = hasher.finalize().into();

    SignatureData {
        signature_hash,
        pubkey,
        msg: msg.to_vec(),
        signature,
    }
}

/// Ed25519 instruction header length (`num_sigs` + padding + 7x u16 offsets).
const ED25519_HEADER_LEN: u16 = 16;

/// Creates an ed25519 program instruction for signature verification.
/// Must be the first instruction in the transaction for `pre_verify_signature` to work.
fn create_ed25519_instruction(signing_key: &SigningKey, msg: &[u8]) -> Instruction {
    let pubkey = signing_key.verifying_key().to_bytes();
    let signature = signing_key.sign(msg).to_bytes();

    let signature_offset: u16 = ED25519_HEADER_LEN;
    let pubkey_offset: u16 = ED25519_HEADER_LEN + 64;
    let message_offset: u16 = ED25519_HEADER_LEN + 64 + 32;
    let same_ix: u16 = 0xFFFF;

    let mut data = Vec::with_capacity((ED25519_HEADER_LEN + 64 + 32) as usize + msg.len());
    data.push(1u8); // num_signatures
    data.push(0); // padding
    data.extend_from_slice(&signature_offset.to_le_bytes());
    data.extend_from_slice(&same_ix.to_le_bytes());
    data.extend_from_slice(&pubkey_offset.to_le_bytes());
    data.extend_from_slice(&same_ix.to_le_bytes());
    data.extend_from_slice(&message_offset.to_le_bytes());
    data.extend_from_slice(&(msg.len() as u16).to_le_bytes());
    data.extend_from_slice(&same_ix.to_le_bytes());
    data.extend_from_slice(&signature);
    data.extend_from_slice(&pubkey);
    data.extend_from_slice(msg);

    Instruction {
        program_id: solana_sdk::ed25519_program::ID,
        accounts: vec![],
        data,
    }
}

/// Creates the `pre_verify_signature` instruction.
fn create_pre_verify_instruction(
    submitter: Pubkey,
    sig_data: SignatureData,
) -> (Instruction, Pubkey) {
    let (sig_verification_pda, _bump) = Pubkey::find_program_address(
        &[SignatureVerification::SEED, &sig_data.signature_hash],
        &ics07_tendermint::ID,
    );
    let (app_state_pda, _) =
        Pubkey::find_program_address(&[ics07_tendermint::AppState::SEED], &ics07_tendermint::ID);
    let (access_manager_pda, _) = Pubkey::find_program_address(
        &[access_manager::state::AccessManager::SEED],
        &access_manager::ID,
    );

    let accounts = ics07_tendermint::accounts::PreVerifySignature {
        instructions_sysvar: ix_sysvar::ID,
        signature_verification: sig_verification_pda,
        app_state: app_state_pda,
        access_manager: access_manager_pda,
        submitter,
        system_program: solana_sdk::system_program::ID,
    };

    let ix_data = ics07_tendermint::instruction::PreVerifySignature {
        signature: sig_data,
    };

    let instruction = Instruction {
        program_id: ics07_tendermint::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data.data(),
    };

    (instruction, sig_verification_pda)
}

async fn get_verification(banks_client: &BanksClient, pda: Pubkey) -> SignatureVerification {
    let account = banks_client
        .get_account(pda)
        .await
        .unwrap()
        .expect("Account not found");
    // Skip the 8-byte Anchor discriminator
    SignatureVerification::deserialize(&mut &account.data[8..]).unwrap()
}

#[rstest]
#[case::normal_message(b"test message for verification".to_vec())]
#[case::empty_message(vec![])]
#[case::large_message((0u8..=255).cycle().take(500).collect())]
#[tokio::test]
async fn test_pre_verify_signature_valid(#[future] ctx: TestContext, #[case] msg: Vec<u8>) {
    let TestContext {
        banks_client,
        payer,
        recent_blockhash,
    } = ctx.await;

    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let sig_data = create_signature_data(&signing_key, &msg);

    let ed25519_ix = create_ed25519_instruction(&signing_key, &msg);
    let (pre_verify_ix, sig_verification_pda) =
        create_pre_verify_instruction(payer.pubkey(), sig_data);

    let tx = Transaction::new_signed_with_payer(
        &[ed25519_ix, pre_verify_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    let verification = get_verification(&banks_client, sig_verification_pda).await;
    assert!(verification.is_valid, "Signature should be valid");
    assert_eq!(verification.submitter, payer.pubkey());
}

enum TamperType {
    Pubkey,
    Message,
    Signature,
}

fn create_tampered_sig_data(
    signing_key: &SigningKey,
    tamper: TamperType,
) -> (SignatureData, Vec<u8>) {
    let msg = b"test message";
    let real_msg = msg.to_vec();

    match tamper {
        TamperType::Pubkey => {
            let signature = signing_key.sign(msg).to_bytes();
            let wrong_key = SigningKey::generate(&mut rand::thread_rng());
            let sig_data =
                create_signature_data_raw(wrong_key.verifying_key().to_bytes(), msg, signature);
            (sig_data, real_msg)
        }
        TamperType::Message => {
            let signature = signing_key.sign(msg).to_bytes();
            let sig_data = create_signature_data_raw(
                signing_key.verifying_key().to_bytes(),
                b"fake message",
                signature,
            );
            (sig_data, real_msg)
        }
        TamperType::Signature => {
            let different_sig = signing_key.sign(b"different message").to_bytes();
            let sig_data = create_signature_data_raw(
                signing_key.verifying_key().to_bytes(),
                msg,
                different_sig,
            );
            (sig_data, real_msg)
        }
    }
}

#[rstest]
#[case::wrong_pubkey(TamperType::Pubkey)]
#[case::wrong_message(TamperType::Message)]
#[case::wrong_signature(TamperType::Signature)]
#[tokio::test]
async fn test_pre_verify_signature_tampered_returns_invalid(
    #[future] ctx: TestContext,
    #[case] tamper: TamperType,
) {
    let TestContext {
        banks_client,
        payer,
        recent_blockhash,
    } = ctx.await;

    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let (sig_data, real_msg) = create_tampered_sig_data(&signing_key, tamper);

    let ed25519_ix = create_ed25519_instruction(&signing_key, &real_msg);
    let (pre_verify_ix, sig_verification_pda) =
        create_pre_verify_instruction(payer.pubkey(), sig_data);

    let tx = Transaction::new_signed_with_payer(
        &[ed25519_ix, pre_verify_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    let verification = get_verification(&banks_client, sig_verification_pda).await;
    assert!(
        !verification.is_valid,
        "Signature should be invalid due to tampered data"
    );
}

#[rstest]
#[case::no_ed25519_instruction(false)]
#[case::ed25519_at_wrong_index(true)]
#[tokio::test]
async fn test_pre_verify_signature_ed25519_position_invalid(
    #[future] ctx: TestContext,
    #[case] include_ed25519: bool,
) {
    let TestContext {
        banks_client,
        payer,
        recent_blockhash,
    } = ctx.await;

    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let msg = b"test message";
    let sig_data = create_signature_data(&signing_key, msg);

    let (pre_verify_ix, sig_verification_pda) =
        create_pre_verify_instruction(payer.pubkey(), sig_data);

    let instructions = if include_ed25519 {
        let dummy_ix = ComputeBudgetInstruction::set_compute_unit_limit(200_000);
        let ed25519_ix = create_ed25519_instruction(&signing_key, msg);
        vec![dummy_ix, ed25519_ix, pre_verify_ix]
    } else {
        vec![pre_verify_ix]
    };

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    let verification = get_verification(&banks_client, sig_verification_pda).await;
    assert!(
        !verification.is_valid,
        "Signature should be invalid due to ed25519 position"
    );
}

#[tokio::test]
async fn test_pre_verify_signature_malformed_ed25519_fails() {
    let TestContext {
        banks_client,
        payer,
        recent_blockhash,
    } = ctx().await;

    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let msg = b"test message";
    let sig_data = create_signature_data(&signing_key, msg);

    let pubkey = signing_key.verifying_key().to_bytes();
    let signature = signing_key.sign(msg).to_bytes();

    let mut data = Vec::with_capacity(16 + 64 + 32 + msg.len());
    data.push(2u8); // num_signatures = 2 (but we only provide 1)
    data.push(0);
    data.extend_from_slice(&16u16.to_le_bytes());
    data.extend_from_slice(&0xFFFFu16.to_le_bytes());
    data.extend_from_slice(&80u16.to_le_bytes());
    data.extend_from_slice(&0xFFFFu16.to_le_bytes());
    data.extend_from_slice(&112u16.to_le_bytes());
    data.extend_from_slice(&(msg.len() as u16).to_le_bytes());
    data.extend_from_slice(&0xFFFFu16.to_le_bytes());
    data.extend_from_slice(&signature);
    data.extend_from_slice(&pubkey);
    data.extend_from_slice(msg);

    let ed25519_ix = Instruction {
        program_id: solana_sdk::ed25519_program::ID,
        accounts: vec![],
        data,
    };

    let (pre_verify_ix, _) = create_pre_verify_instruction(payer.pubkey(), sig_data);

    let tx = Transaction::new_signed_with_payer(
        &[ed25519_ix, pre_verify_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    assert!(
        banks_client.process_transaction(tx).await.is_err(),
        "Malformed ed25519 instruction should fail at precompile"
    );
}

#[tokio::test]
async fn test_pre_verify_signature_duplicate_pda_fails() {
    let TestContext {
        mut banks_client,
        payer,
        recent_blockhash,
    } = ctx().await;

    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let msg = b"test message";
    let sig_data = create_signature_data(&signing_key, msg);

    let ed25519_ix = create_ed25519_instruction(&signing_key, msg);
    let (pre_verify_ix, _) = create_pre_verify_instruction(payer.pubkey(), sig_data.clone());

    let tx1 = Transaction::new_signed_with_payer(
        &[ed25519_ix.clone(), pre_verify_ix.clone()],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx1).await.unwrap();

    let new_blockhash = banks_client
        .get_new_latest_blockhash(&recent_blockhash)
        .await
        .unwrap();

    let (pre_verify_ix2, _) = create_pre_verify_instruction(payer.pubkey(), sig_data);
    let tx2 = Transaction::new_signed_with_payer(
        &[ed25519_ix, pre_verify_ix2],
        Some(&payer.pubkey()),
        &[&payer],
        new_blockhash,
    );

    assert!(
        banks_client.process_transaction(tx2).await.is_err(),
        "Second verification with same signature should fail"
    );
}

enum PoisonField {
    Signature,
    Pubkey,
    Message,
}

#[rstest]
#[case::signature_from_other_ix(PoisonField::Signature)]
#[case::pubkey_from_other_ix(PoisonField::Pubkey)]
#[case::message_from_other_ix(PoisonField::Message)]
#[tokio::test]
async fn test_cross_instruction_index_rejected(
    #[future] ctx: TestContext,
    #[case] field_to_poison: PoisonField,
) {
    let TestContext {
        banks_client,
        payer,
        recent_blockhash,
    } = ctx.await;

    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let msg = b"test message";
    let sig_data = create_signature_data(&signing_key, msg);

    // Build a valid ed25519 instruction but tamper with one instruction_index
    // field to point to instruction index 1 instead of 0xFFFF (inline).
    let ed25519_ix = {
        let pubkey = signing_key.verifying_key().to_bytes();
        let signature = signing_key.sign(msg).to_bytes();

        let signature_offset: u16 = ED25519_HEADER_LEN;
        let pubkey_offset: u16 = ED25519_HEADER_LEN + 64;
        let message_offset: u16 = ED25519_HEADER_LEN + 64 + 32;
        let same_ix: u16 = 0xFFFF;
        // Points to a different instruction in the transaction
        let cross_ix: u16 = 1;

        let sig_ix_idx = if matches!(field_to_poison, PoisonField::Signature) {
            cross_ix
        } else {
            same_ix
        };
        let pk_ix_idx = if matches!(field_to_poison, PoisonField::Pubkey) {
            cross_ix
        } else {
            same_ix
        };
        let msg_ix_idx = if matches!(field_to_poison, PoisonField::Message) {
            cross_ix
        } else {
            same_ix
        };

        let mut data = Vec::with_capacity(16 + 64 + 32 + msg.len());
        data.push(1u8); // num_signatures
        data.push(0); // padding
        data.extend_from_slice(&signature_offset.to_le_bytes());
        data.extend_from_slice(&sig_ix_idx.to_le_bytes());
        data.extend_from_slice(&pubkey_offset.to_le_bytes());
        data.extend_from_slice(&pk_ix_idx.to_le_bytes());
        data.extend_from_slice(&message_offset.to_le_bytes());
        data.extend_from_slice(&(msg.len() as u16).to_le_bytes());
        data.extend_from_slice(&msg_ix_idx.to_le_bytes());
        data.extend_from_slice(&signature);
        data.extend_from_slice(&pubkey);
        data.extend_from_slice(msg);

        Instruction {
            program_id: solana_sdk::ed25519_program::ID,
            accounts: vec![],
            data,
        }
    };

    let (pre_verify_ix, sig_verification_pda) =
        create_pre_verify_instruction(payer.pubkey(), sig_data);

    let tx = Transaction::new_signed_with_payer(
        &[ed25519_ix, pre_verify_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // The transaction may succeed (precompile might reject, or our code marks invalid).
    // What matters is the PDA must NOT be marked valid.
    if matches!(banks_client.process_transaction(tx).await, Ok(())) {
        let verification = get_verification(&banks_client, sig_verification_pda).await;
        assert!(
            !verification.is_valid,
            "Cross-instruction index attack must not mark signature as valid"
        );
    } else {
        // Transaction rejected entirely — also acceptable
    }
}

/// Attack scenario: attacker passes a valid ed25519 signature for key B but uses
/// a `signature_hash` computed from key A's data, trying to create a valid PDA
/// for a signature that was never actually verified.
#[tokio::test]
async fn test_mismatched_signature_hash_rejected() {
    let TestContext {
        banks_client,
        payer,
        recent_blockhash,
    } = ctx().await;

    let key_a = SigningKey::generate(&mut rand::thread_rng());
    let key_b = SigningKey::generate(&mut rand::thread_rng());
    let msg = b"test message";

    let target_sig_data = create_signature_data(&key_a, msg);

    let attacker_sig_data = SignatureData {
        signature_hash: target_sig_data.signature_hash,
        pubkey: key_b.verifying_key().to_bytes(),
        msg: msg.to_vec(),
        signature: key_b.sign(msg).to_bytes(),
    };

    let ed25519_ix = create_ed25519_instruction(&key_b, msg);
    let (pre_verify_ix, _) = create_pre_verify_instruction(payer.pubkey(), attacker_sig_data);

    let tx = Transaction::new_signed_with_payer(
        &[ed25519_ix, pre_verify_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    assert!(
        banks_client.process_transaction(tx).await.is_err(),
        "Mismatched signature_hash should be rejected"
    );
}
