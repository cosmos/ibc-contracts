// SPDX-License-Identifier: Apache-2.0

use crate::state::ConsensusStateStore;
use crate::types::ClientState;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;

/// Returns the current status of the light client (active, frozen or expired) via return data.
#[derive(Accounts)]
pub struct ClientStatus<'info> {
    /// PDA holding the light client configuration; checked for frozen state and trusting period.
    #[account(
        seeds = [ClientState::SEED],
        bump
    )]
    pub client_state: Account<'info, ClientState>,
    /// Consensus state at the latest height; its timestamp is compared against the
    /// current clock to determine whether the client has expired.
    #[account(
        seeds = [
            ConsensusStateStore::SEED,
            &client_state.latest_height.revision_height.to_le_bytes()
        ],
        bump
    )]
    pub consensus_state: Account<'info, ConsensusStateStore>,
}

pub fn client_status(ctx: Context<ClientStatus>) -> Result<()> {
    let client_state = &ctx.accounts.client_state;
    let status = if client_state.is_frozen() {
        ics25_handler::ClientStatus::Frozen
    } else {
        let clock = Clock::get()?;
        let consensus_ts_secs =
            crate::nanos_to_secs(ctx.accounts.consensus_state.consensus_state.timestamp) as i64;
        let elapsed = clock.unix_timestamp.saturating_sub(consensus_ts_secs);
        if elapsed > client_state.trusting_period as i64 {
            ics25_handler::ClientStatus::Expired
        } else {
            ics25_handler::ClientStatus::Active
        }
    };
    set_return_data(&[status.into()]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::state::ConsensusStateStore;
    use crate::test_helpers::chunk_test_utils::{
        derive_client_state_pda, derive_consensus_state_pda,
    };
    use crate::test_helpers::fixtures::*;
    use crate::test_helpers::PROGRAM_BINARY_PATH;
    use crate::types::IbcHeight;
    use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
    use anchor_lang::AccountSerialize;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_sdk::account::Account;

    fn setup_client_status_test(
        frozen: bool,
    ) -> (Instruction, Vec<(solana_sdk::pubkey::Pubkey, Account)>) {
        let fixture = load_membership_verification_fixture("verify_membership_key_0");
        let mut client_state = decode_client_state_from_hex(&fixture.client_state_hex);
        let consensus_state = decode_consensus_state_from_hex(&fixture.consensus_state_hex);

        if frozen {
            client_state.frozen_height = IbcHeight {
                revision_number: 0,
                revision_height: 1,
            };
        }

        let client_state_pda = derive_client_state_pda();
        let height = client_state.latest_height.revision_height;
        let consensus_state_pda = derive_consensus_state_pda(height);

        let mut client_data = vec![];
        client_state.try_serialize(&mut client_data).unwrap();

        let consensus_state_store = ConsensusStateStore {
            height,
            consensus_state,
        };
        let mut consensus_data = vec![];
        consensus_state_store
            .try_serialize(&mut consensus_data)
            .unwrap();

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(client_state_pda, false),
                AccountMeta::new_readonly(consensus_state_pda, false),
            ],
            data: crate::instruction::ClientStatus {}.data(),
        };

        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: client_data,
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                consensus_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: consensus_data,
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        (instruction, accounts)
    }

    fn run_client_status_test(frozen: bool, clock_timestamp: i64) -> Vec<u8> {
        let (instruction, accounts) = setup_client_status_test(frozen);
        let mut mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        mollusk.sysvars.clock.unix_timestamp = clock_timestamp;
        mollusk.sysvars.clock.slot = 1;
        let checks = vec![Check::success()];
        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
        result.return_data
    }

    fn consensus_ts_secs(fixture_name: &str) -> i64 {
        let fixture = load_membership_verification_fixture(fixture_name);
        let cs = decode_consensus_state_from_hex(&fixture.consensus_state_hex);
        crate::nanos_to_secs(cs.timestamp) as i64
    }

    #[test]
    fn test_client_status_active() {
        // Set clock just after consensus state timestamp (within trusting period)
        let clock_ts = consensus_ts_secs("verify_membership_key_0") + 1;
        let return_data = run_client_status_test(false, clock_ts);
        assert_eq!(
            return_data,
            vec![u8::from(ics25_handler::ClientStatus::Active)]
        );
    }

    #[test]
    fn test_client_status_frozen() {
        let clock_ts = consensus_ts_secs("verify_membership_key_0") + 1;
        let return_data = run_client_status_test(true, clock_ts);
        assert_eq!(
            return_data,
            vec![u8::from(ics25_handler::ClientStatus::Frozen)]
        );
    }

    #[test]
    fn test_client_status_expired() {
        let fixture = load_membership_verification_fixture("verify_membership_key_0");
        let client_state = decode_client_state_from_hex(&fixture.client_state_hex);
        let cs = decode_consensus_state_from_hex(&fixture.consensus_state_hex);
        let cs_secs = crate::nanos_to_secs(cs.timestamp) as i64;
        // Set clock well past the trusting period
        let clock_ts = cs_secs + client_state.trusting_period as i64 + 1;
        let return_data = run_client_status_test(false, clock_ts);
        assert_eq!(
            return_data,
            vec![u8::from(ics25_handler::ClientStatus::Expired)]
        );
    }
}
