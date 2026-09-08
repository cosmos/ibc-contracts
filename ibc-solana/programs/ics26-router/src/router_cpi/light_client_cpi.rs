// SPDX-License-Identifier: Apache-2.0

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use crate::errors::RouterError;
use crate::state::Client;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::{get_return_data, invoke};
use ics25_handler::{discriminators, MembershipMsg, NonMembershipMsg};

/// Light client CPI wrapper
pub struct LightClientCpi<'a> {
    client: &'a Client,
}

impl<'a> LightClientCpi<'a> {
    pub const fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub fn verify_membership<'b>(
        &self,
        light_client_program: &AccountInfo<'b>,
        client_state: &AccountInfo<'b>,
        consensus_state: &AccountInfo<'b>,
        msg: MembershipMsg,
    ) -> Result<()> {
        self.validate_client(light_client_program)?;

        let ix_data = Self::build_instruction_data(discriminators::VERIFY_MEMBERSHIP, msg)?;
        self.invoke_instruction(light_client_program, client_state, consensus_state, ix_data)?;

        Ok(())
    }

    /// Query the light client's status (active, frozen, expired)
    pub fn client_status<'b>(
        &self,
        light_client_program: &AccountInfo<'b>,
        client_state: &AccountInfo<'b>,
        consensus_state: &AccountInfo<'b>,
    ) -> Result<ics25_handler::ClientStatus> {
        self.validate_client(light_client_program)?;

        let ix_data = discriminators::CLIENT_STATUS.to_vec();
        self.invoke_instruction(light_client_program, client_state, consensus_state, ix_data)?;

        let (program_id, data) = get_return_data().ok_or(RouterError::InvalidAppResponse)?;
        require_eq!(
            program_id,
            self.client.client_program_id,
            RouterError::InvalidLightClientProgram
        );
        require!(!data.is_empty(), RouterError::InvalidAppResponse);

        ics25_handler::ClientStatus::try_from(data[0])
            .map_err(|_| RouterError::InvalidAppResponse.into())
    }

    /// Verify non-membership (absence) of a value at a given path
    /// Returns the timestamp from the consensus state at the proof height
    pub fn verify_non_membership<'b>(
        &self,
        light_client_program: &AccountInfo<'b>,
        client_state: &AccountInfo<'b>,
        consensus_state: &AccountInfo<'b>,
        msg: NonMembershipMsg,
    ) -> Result<u64> {
        self.validate_client(light_client_program)?;

        let ix_data = Self::build_instruction_data(discriminators::VERIFY_NON_MEMBERSHIP, msg)?;
        self.invoke_instruction(light_client_program, client_state, consensus_state, ix_data)?;

        // Extract timestamp from return data
        self.get_timestamp_from_return_data()
    }

    fn validate_client(&self, light_client_program: &AccountInfo) -> Result<()> {
        require!(
            light_client_program.key() == self.client.client_program_id,
            RouterError::InvalidLightClientProgram
        );

        require!(self.client.active, RouterError::ClientNotActive);

        Ok(())
    }

    fn build_instruction_data<T: AnchorSerialize>(
        discriminator: [u8; 8],
        msg: T,
    ) -> Result<Vec<u8>> {
        let mut ix_data = Vec::new();
        ix_data.extend_from_slice(&discriminator);
        msg.serialize(&mut ix_data)?;
        Ok(ix_data)
    }

    fn invoke_instruction<'b>(
        &self,
        light_client_program: &AccountInfo<'b>,
        client_state: &AccountInfo<'b>,
        consensus_state: &AccountInfo<'b>,
        data: Vec<u8>,
    ) -> Result<()> {
        require_eq!(
            *client_state.owner,
            self.client.client_program_id,
            RouterError::InvalidAccountOwner
        );

        // Build the instruction with standard account layout
        // All light clients must accept: [client_state, consensus_state]
        let ix = Instruction::new_with_bytes(
            self.client.client_program_id,
            &data,
            vec![
                AccountMeta::new_readonly(client_state.key(), false),
                AccountMeta::new_readonly(consensus_state.key(), false),
            ],
        );

        let account_infos = vec![
            client_state.to_account_info(),
            consensus_state.to_account_info(),
            light_client_program.to_account_info(),
        ];

        invoke(&ix, &account_infos)?;

        Ok(())
    }

    /// Extract timestamp from light client return data
    fn get_timestamp_from_return_data(&self) -> Result<u64> {
        let (program_id, data) = get_return_data().ok_or(RouterError::InvalidAppResponse)?;
        require_eq!(
            program_id,
            self.client.client_program_id,
            RouterError::InvalidLightClientProgram
        );

        require!(
            data.len() >= ANCHOR_DISCRIMINATOR_SIZE,
            RouterError::InvalidAppResponse
        );

        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[..ANCHOR_DISCRIMINATOR_SIZE]);
        Ok(u64::from_le_bytes(bytes))
    }
}
