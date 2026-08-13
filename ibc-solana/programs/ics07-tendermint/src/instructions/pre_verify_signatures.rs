// SPDX-License-Identifier: Apache-2.0

use crate::types::AppState;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as ix_sysvar;
use solana_ibc_types::ics07::SignatureData;
use solana_sdk_ids::ed25519_program;

/// Verifies an Ed25519 signature by introspecting a preceding `Ed25519Program` instruction
/// and stores the result in a PDA for later use during header or misbehaviour assembly.
#[derive(Accounts)]
#[instruction(signature: SignatureData)]
pub struct PreVerifySignature<'info> {
    /// Instructions sysvar used to load and introspect the preceding Ed25519 verify instruction.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

    /// PDA that stores the Ed25519 verification outcome, keyed by the signature hash.
    #[account(
        init,
        payer = submitter,
        space = 8 + crate::state::SignatureVerification::INIT_SPACE,
        seeds = [
            crate::state::SignatureVerification::SEED,
            &signature.signature_hash
        ],
        bump
    )]
    pub signature_verification: Account<'info, crate::state::SignatureVerification>,

    /// PDA holding program-level settings; provides the `access_manager` address for role checks.
    #[account(
        seeds = [AppState::SEED],
        bump
    )]
    pub app_state: Account<'info, AppState>,

    /// Access-manager PDA used to verify the submitter holds the relayer role.
    /// CHECK: Validated by seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// Relayer that signs the transaction and pays for the verification PDA creation.
    #[account(mut)]
    pub submitter: Signer<'info>,
    /// Required by Anchor for PDA creation via the System Program.
    pub system_program: Program<'info, System>,
}

pub fn pre_verify_signature<'info>(
    ctx: Context<'_, '_, '_, 'info, PreVerifySignature<'info>>,
    signature: SignatureData,
) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.submitter,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let expected_hash =
        solana_sha256_hasher::hashv(&[&signature.pubkey, &signature.msg, &signature.signature]);
    require!(
        signature.signature_hash == expected_hash.to_bytes(),
        crate::error::ErrorCode::InvalidAccountData
    );

    let ix_sysvar = &ctx.accounts.instructions_sysvar;

    let is_valid = verify_ed25519_from_sysvar(
        ix_sysvar,
        &signature.pubkey,
        &signature.msg,
        &signature.signature,
    )?;

    ctx.accounts.signature_verification.is_valid = is_valid;
    ctx.accounts.signature_verification.submitter = ctx.accounts.submitter.key();

    Ok(())
}

const ED25519_NUM_SIGNATURES_OFFSET: usize = 0;
const ED25519_SIGNATURE_IX_INDEX_OFFSET: usize = 4;
const ED25519_PUBKEY_IX_INDEX_OFFSET: usize = 8;
const ED25519_MESSAGE_OFFSET: usize = 10;
const ED25519_MESSAGE_SIZE_OFFSET: usize = 12;
const ED25519_MESSAGE_IX_INDEX_OFFSET: usize = 14;
const ED25519_HEADER_SIZE: usize = 16;

fn verify_ed25519_from_sysvar(
    ix_sysvar: &AccountInfo,
    pubkey: &[u8; 32],
    msg: &[u8],
    signature: &[u8; 64],
) -> Result<bool> {
    const ED25519_IX_INDEX: usize = 0;
    let ix = ix_sysvar::load_instruction_at_checked(ED25519_IX_INDEX, ix_sysvar)?;

    if ix.program_id != ed25519_program::ID
        || ix.data.len() < ED25519_HEADER_SIZE + 96
        || ix.data[ED25519_NUM_SIGNATURES_OFFSET] != 1
    {
        return Ok(false);
    }

    if !all_instruction_indices_inline(&ix.data) {
        return Ok(false);
    }

    // Check msg size before loading offsets
    let msg_size = u16::from_le_bytes([
        ix.data[ED25519_MESSAGE_SIZE_OFFSET],
        ix.data[ED25519_MESSAGE_SIZE_OFFSET + 1],
    ]) as usize;

    if msg_size != msg.len() {
        return Ok(false);
    }

    let offsets = load_offsets(&ix.data);

    // Bounds check
    if offsets.signature + 64 > ix.data.len()
        || offsets.pubkey + 32 > ix.data.len()
        || offsets.msg + msg_size > ix.data.len()
    {
        return Ok(false);
    }

    Ok(&ix.data[offsets.pubkey..offsets.pubkey + 32] == pubkey
        && &ix.data[offsets.signature..offsets.signature + 64] == signature
        && &ix.data[offsets.msg..offsets.msg + msg_size] == msg)
}

#[inline]
const fn load_offsets(data: &[u8]) -> Offsets {
    Offsets {
        signature: u16::from_le_bytes([data[2], data[3]]) as usize,
        pubkey: u16::from_le_bytes([data[6], data[7]]) as usize,
        msg: u16::from_le_bytes([data[10], data[11]]) as usize,
    }
}

struct Offsets {
    signature: usize,
    pubkey: usize,
    msg: usize,
}

/// Returns `true` only when all three `*_instruction_index` fields equal
/// `u16::MAX`, meaning the Ed25519 precompile reads signature, pubkey, and
/// message from its own instruction data (not from another instruction).
const fn all_instruction_indices_inline(data: &[u8]) -> bool {
    let sig_ix = u16::from_le_bytes([
        data[ED25519_SIGNATURE_IX_INDEX_OFFSET],
        data[ED25519_SIGNATURE_IX_INDEX_OFFSET + 1],
    ]);
    let pk_ix = u16::from_le_bytes([
        data[ED25519_PUBKEY_IX_INDEX_OFFSET],
        data[ED25519_PUBKEY_IX_INDEX_OFFSET + 1],
    ]);
    let msg_ix = u16::from_le_bytes([
        data[ED25519_MESSAGE_IX_INDEX_OFFSET],
        data[ED25519_MESSAGE_IX_INDEX_OFFSET + 1],
    ]);

    // When `instruction_index != u16::MAX`, the Agave runtime reads data from
    // a different instruction in the transaction, enabling a substitution attack:
    //   - Verify logic: <https://github.com/anza-xyz/agave/blob/95542233cd95309fab114fe9dcbbbd63a06e5390/precompiles/src/ed25519.rs#L51-L76>
    //   - Branch on `u16::MAX`<https://github.com/anza-xyz/agave/blob/95542233cd95309fab114fe9dcbbbd63a06e5390/precompiles/src/ed25519.rs#L88-L96>96
    sig_ix == u16::MAX && pk_ix == u16::MAX && msg_ix == u16::MAX
}

#[cfg(test)]
mod integration_tests {
    use crate::test_helpers::*;
    use anchor_lang::InstructionData;
    use solana_ibc_types::ics07::SignatureData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn build_pre_verify_signature_ix(submitter: Pubkey) -> Instruction {
        let signature = SignatureData {
            signature_hash: [1u8; 32],
            pubkey: [2u8; 32],
            msg: vec![3u8; 32],
            signature: [4u8; 64],
        };

        let (sig_verification_pda, _) = Pubkey::find_program_address(
            &[
                crate::state::SignatureVerification::SEED,
                &signature.signature_hash,
            ],
            &crate::ID,
        );
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(sig_verification_pda, false),
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(submitter, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data: crate::instruction::PreVerifySignature { signature }.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_without_relayer_role_rejected() {
        let relayer = Keypair::new();
        let non_relayer = Keypair::new();
        let mut pt = setup_program_test_with_relayer(&relayer.pubkey());
        fund_account(&mut pt, &non_relayer.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_pre_verify_signature_ix(non_relayer.pubkey());

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_relayer],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_cpi_rejected() {
        let relayer = Keypair::new();
        let pt = setup_program_test_with_relayer(&relayer.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_pre_verify_signature_ix(relayer.pubkey());
        let wrapped_ix = wrap_in_test_cpi_proxy(relayer.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &relayer],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
