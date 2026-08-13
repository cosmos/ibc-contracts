// SPDX-License-Identifier: Apache-2.0

//! Client operations - create/update client and signature verification.

use anyhow::{Context, Result};
use ibc_client_tendermint::types::Header as TmHeader;
use solana_sdk::{
    ed25519_instruction::{Ed25519SignatureOffsets, DATA_START},
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use tendermint::{chain::Id as ChainId, vote::CanonicalVote};
use tendermint_proto::Protobuf;

use proof_api_lib::utils::solana_v0_tx::{
    derive_alt_address, extend_compute_ix, extend_compute_ix_with_heap,
};
use solana_ibc_sdk::access_manager::instructions as access_manager_instructions;
use solana_ibc_sdk::ics07_tendermint::accounts::ClientState;
use solana_ibc_sdk::ics07_tendermint::instructions::{
    AssembleAndUpdateClient, AssembleAndUpdateClientAccounts, AssembleAndUpdateClientArgs,
    CleanupIncompleteUpload, CleanupIncompleteUploadAccounts, Initialize, InitializeAccounts,
    InitializeArgs, PreVerifySignature, PreVerifySignatureAccounts, UploadHeaderChunk,
    UploadHeaderChunkAccounts,
};
use solana_ibc_sdk::ics07_tendermint::types::{ConsensusState, SignatureData, UploadChunkParams};
use solana_ibc_sdk::pda::ics07_tendermint::{header_chunk_pda, sig_verify_pda};

impl super::TxBuilder {
    pub(crate) fn build_create_client_instruction(
        &self,
        latest_height: u64,
        client_state: &ClientState,
        consensus_state: &ConsensusState,
        access_manager: Pubkey,
        authority: Pubkey,
    ) -> Instruction {
        // For create_client, we use the default ICS07 Tendermint light client.
        let solana_ics07_program_id: Pubkey = solana_ibc_constants::ICS07_TENDERMINT_ID
            .parse()
            .expect("Invalid ICS07_TENDERMINT_ID constant");

        let (program_data_pda, _) = Pubkey::find_program_address(
            &[solana_ics07_program_id.as_ref()],
            &solana_sdk::bpf_loader_upgradeable::id(),
        );

        Initialize::builder(&solana_ics07_program_id)
            .accounts(InitializeAccounts {
                payer: self.fee_payer,
                program_data: program_data_pda,
                authority,
                revision_height: latest_height,
            })
            .args(&InitializeArgs {
                client_state: client_state.clone(),
                consensus_state: consensus_state.clone(),
                access_manager,
            })
            .build()
    }

    pub(crate) fn build_assemble_and_update_client_tx(
        &self,
        target_height: u64,
        trusted_height: u64,
        total_chunks: u8,
        signature_data: &[SignatureData],
        alt_config: Option<(u64, Vec<Pubkey>)>,
        solana_ics07_program_id: Pubkey,
    ) -> Result<Vec<u8>> {
        let access_manager_program_id = self.resolve_access_manager_program_id()?;
        let (access_manager, _) =
            access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);

        let chunk_iter = (0..total_chunks).map(|chunk_index| {
            let (chunk_pda, _) = header_chunk_pda(
                &self.fee_payer,
                target_height,
                chunk_index,
                &solana_ics07_program_id,
            );
            AccountMeta::new(chunk_pda, false)
        });

        let sig_iter = signature_data.iter().map(|sig_data| {
            let (sig_verify_pda, _) =
                sig_verify_pda(&sig_data.signature_hash, &solana_ics07_program_id);
            AccountMeta::new_readonly(sig_verify_pda, false)
        });

        let ix = AssembleAndUpdateClient::builder(&solana_ics07_program_id)
            .accounts(AssembleAndUpdateClientAccounts {
                access_manager,
                submitter: self.fee_payer,
                trusted_height,
                target_height,
            })
            .args(&AssembleAndUpdateClientArgs {
                target_height,
                chunk_count: total_chunks,
                trusted_height,
            })
            .remaining_accounts(chunk_iter.chain(sig_iter))
            .build();

        let mut instructions = extend_compute_ix_with_heap();
        instructions.push(ix);

        match alt_config {
            Some((slot, addresses)) => {
                let (alt_address, _) = derive_alt_address(slot, self.fee_payer);
                self.create_tx_bytes_with_alt(&instructions, alt_address, addresses)
            }
            None => self.create_tx_bytes(&instructions),
        }
    }

    pub(crate) fn build_cleanup_tx(
        &self,
        target_height: u64,
        total_chunks: u8,
        signature_data: &[SignatureData],
        solana_ics07_program_id: Pubkey,
    ) -> Result<Vec<u8>> {
        let chunk_iter = (0..total_chunks).map(|chunk_index| {
            let (chunk_pda, _) = header_chunk_pda(
                &self.fee_payer,
                target_height,
                chunk_index,
                &solana_ics07_program_id,
            );
            AccountMeta::new(chunk_pda, false)
        });

        let sig_iter = signature_data.iter().map(|sig_data| {
            let (sig_verify_pda, _) =
                sig_verify_pda(&sig_data.signature_hash, &solana_ics07_program_id);
            AccountMeta::new(sig_verify_pda, false)
        });

        let instruction = CleanupIncompleteUpload::builder(&solana_ics07_program_id)
            .accounts(CleanupIncompleteUploadAccounts {
                submitter: self.fee_payer,
            })
            .remaining_accounts(chunk_iter.chain(sig_iter))
            .build();

        let mut instructions = extend_compute_ix();
        instructions.push(instruction);

        self.create_tx_bytes(&instructions)
    }

    pub(crate) fn build_upload_header_chunk_instruction(
        &self,
        target_height: u64,
        chunk_index: u8,
        chunk_data: Vec<u8>,
        solana_ics07_program_id: Pubkey,
    ) -> Result<Instruction> {
        let params = UploadChunkParams {
            target_height,
            chunk_index,
            chunk_data,
        };

        let access_manager_program_id = self.resolve_access_manager_program_id()?;
        let (access_manager, _) =
            access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);
        let (chunk_pda, _) = header_chunk_pda(
            &self.fee_payer,
            target_height,
            chunk_index,
            &solana_ics07_program_id,
        );

        Ok(UploadHeaderChunk::builder(&solana_ics07_program_id)
            .accounts(UploadHeaderChunkAccounts {
                chunk: chunk_pda,
                access_manager,
                submitter: self.fee_payer,
            })
            .args(&params)
            .build())
    }

    pub(crate) fn build_chunk_transactions(
        &self,
        chunks: &[Vec<u8>],
        target_height: u64,
        light_client_program_id: Pubkey,
    ) -> Result<Vec<Vec<u8>>> {
        chunks
            .iter()
            .enumerate()
            .map(|(index, chunk_data)| {
                let chunk_index = u8::try_from(index)
                    .map_err(|_| anyhow::anyhow!("Chunk index {index} exceeds u8 max"))?;
                let upload_ix = self.build_upload_header_chunk_instruction(
                    target_height,
                    chunk_index,
                    chunk_data.clone(),
                    light_client_program_id,
                )?;
                self.create_tx_bytes(&[upload_ix])
            })
            .collect()
    }

    /// Extracts signature data from Protobuf Tendermint Header for Ed25519 pre-verification
    pub(crate) fn extract_signature_data_from_header(
        header: &ibc_client_tendermint::types::Header,
        chain_id: &str,
    ) -> Result<Vec<SignatureData>> {
        use tendermint::validator::Info as ValidatorInfo;

        let commit = &header.signed_header.commit;
        let validators = &header.validator_set.validators();

        let chain_id =
            ChainId::try_from(chain_id.to_string()).context("Failed to parse chain ID")?;

        let mut signature_data_vec = Vec::new();
        let mut seen_hashes = std::collections::HashSet::new();
        let mut duplicates_skipped = 0;

        for commit_sig in &commit.signatures {
            let (validator_address, timestamp, signature_opt) = match commit_sig {
                tendermint::block::CommitSig::BlockIdFlagCommit {
                    validator_address,
                    timestamp,
                    signature,
                }
                | tendermint::block::CommitSig::BlockIdFlagNil {
                    validator_address,
                    timestamp,
                    signature,
                } => (validator_address, timestamp, signature),
                tendermint::block::CommitSig::BlockIdFlagAbsent => continue,
            };

            let Some(signature_bytes) = signature_opt else {
                continue;
            };

            let validator: &ValidatorInfo = validators
                .iter()
                .find(|v| v.address == *validator_address)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Validator address {validator_address:?} not found in validator set",
                    )
                })?;

            let pubkey = match &validator.pub_key {
                tendermint::PublicKey::Ed25519(key) => key.as_bytes(),
                _ => {
                    anyhow::bail!("Only Ed25519 keys are supported for signature verification");
                }
            };

            let canonical_vote = CanonicalVote {
                vote_type: tendermint::vote::Type::Precommit,
                height: commit.height,
                round: commit.round,
                block_id: Some(commit.block_id),
                timestamp: Some(*timestamp),
                chain_id: chain_id.clone(),
            };

            let sign_bytes = <CanonicalVote as Protobuf<
                tendermint_proto::v0_38::types::CanonicalVote,
            >>::encode_length_delimited_vec(canonical_vote);

            let signature: [u8; 64] = signature_bytes
                .as_bytes()
                .try_into()
                .context("Signature must be 64 bytes")?;

            let signature_hash =
                solana_sdk::hash::hashv(&[pubkey, sign_bytes.as_slice(), &signature]).to_bytes();

            if !seen_hashes.insert(signature_hash) {
                duplicates_skipped += 1;
                continue;
            }

            signature_data_vec.push(SignatureData {
                signature_hash,
                pubkey: pubkey.try_into().context("Public key must be 32 bytes")?,
                msg: sign_bytes,
                signature,
            });
        }

        tracing::debug!(
            "Extracted {} signatures ({} dups skipped)",
            signature_data_vec.len(),
            duplicates_skipped
        );

        Ok(signature_data_vec)
    }

    /// Select minimal signatures to meet 2/3 threshold
    pub(crate) fn select_minimal_signatures(
        signature_data: &[SignatureData],
        header: &TmHeader,
        trust_numerator: u64,
        trust_denominator: u64,
    ) -> Result<Vec<SignatureData>> {
        let untrusted_validator_set = &header.validator_set;
        let untrusted_total_power: u64 = untrusted_validator_set.total_voting_power().into();
        let untrusted_required_power = (untrusted_total_power * 2) / 3;

        let mut accumulated_power = 0u64;
        let mut selected = Vec::new();

        for validator in untrusted_validator_set.validators() {
            let pubkey_bytes = validator.pub_key.to_bytes();

            if let Some(sig_data) = signature_data.iter().find(|sig| pubkey_bytes == sig.pubkey) {
                accumulated_power += validator.power();
                selected.push(sig_data.clone());

                if accumulated_power >= untrusted_required_power {
                    tracing::debug!("Selected {} sigs for 2/3 threshold", selected.len());
                    break;
                }
            }
        }

        if accumulated_power < untrusted_required_power {
            anyhow::bail!(
                "Insufficient voting power: {accumulated_power} < {untrusted_required_power} required",
            );
        }

        let trusted_validator_set = &header.trusted_next_validator_set;
        let trusted_total_power: u64 = trusted_validator_set.total_voting_power().into();
        let trusted_required_power = (trusted_total_power * trust_numerator) / trust_denominator;

        let mut trusted_power = 0u64;
        let selected_pubkeys: std::collections::HashSet<_> =
            selected.iter().map(|s| s.pubkey.as_slice()).collect();

        for validator in trusted_validator_set.validators() {
            if selected_pubkeys.contains(validator.pub_key.to_bytes().as_slice()) {
                trusted_power += validator.power();
            }
        }

        if trusted_power < trusted_required_power {
            anyhow::bail!(
                "Selection fails trusted threshold: {trusted_power} < {trusted_required_power} required",
            );
        }

        Ok(selected)
    }

    /// Builds a single signature pre-verification transaction
    /// NOTE: Can't use `solana_sdk` `ed25519_instruction` because it requires tx signing which we do
    /// in the grpc caller, not here
    pub(crate) fn build_pre_verify_signature_transaction(
        &self,
        sig_data: &SignatureData,
        solana_ics07_program_id: Pubkey,
    ) -> Result<Vec<u8>> {
        let mut instruction_data = vec![
            1u8, // number of signatures
            0u8, // padding
        ];

        let data_start = u16::try_from(DATA_START).expect("DATA_START (16) must fit in u16");
        let pubkey_offset = data_start;
        let signature_offset = data_start + 32;
        let message_data_offset = data_start + 32 + 64;
        let message_data_size = u16::try_from(sig_data.msg.len())
            .context("CanonicalVote message exceeds 65,535 bytes (Ed25519 instruction limit)")?;

        let offsets = Ed25519SignatureOffsets {
            signature_offset,
            signature_instruction_index: u16::MAX,
            public_key_offset: pubkey_offset,
            public_key_instruction_index: u16::MAX,
            message_data_offset,
            message_data_size,
            message_instruction_index: u16::MAX,
        };

        #[allow(clippy::borrow_as_ptr)]
        let offsets_bytes =
            unsafe { std::slice::from_raw_parts((&raw const offsets).cast::<u8>(), 14) };
        instruction_data.extend_from_slice(offsets_bytes);

        instruction_data.extend_from_slice(&sig_data.pubkey);
        instruction_data.extend_from_slice(&sig_data.signature);
        instruction_data.extend_from_slice(&sig_data.msg);

        let ed25519_ix = Instruction {
            program_id: solana_sdk::ed25519_program::ID,
            accounts: vec![],
            data: instruction_data,
        };

        let (sig_verify_pda, _) =
            sig_verify_pda(&sig_data.signature_hash, &solana_ics07_program_id);

        let access_manager_program_id = self.resolve_access_manager_program_id()?;
        let (access_manager, _) =
            access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);

        let pre_verify_ix = PreVerifySignature::builder(&solana_ics07_program_id)
            .accounts(PreVerifySignatureAccounts {
                signature_verification: sig_verify_pda,
                access_manager,
                submitter: self.fee_payer,
            })
            .args(sig_data)
            .build();

        let tx_bytes = self.create_tx_bytes(&[ed25519_ix, pre_verify_ix])?;

        Ok(tx_bytes)
    }
}
