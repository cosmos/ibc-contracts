// SPDX-License-Identifier: Apache-2.0

use access_manager::AccessManagerState;
use anchor_lang::prelude::*;
use solana_ibc_types::{packet_acknowledgement_commitment_bytes32, MAX_CLIENT_ID_LENGTH};

/// Account schema version
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
pub enum AccountVersion {
    V1,
}

/// Global ICS27 General Message Passing (GMP) application state.
///
/// Singleton PDA that stores the pause flag, access manager reference
/// and PDA bump seed. The ICS26 router CPIs into this program to deliver
/// received packets; this account gates whether the app accepts new
/// cross-chain calls and controls admin operations via the access manager.
#[account]
#[derive(InitSpace)]
pub struct GMPAppState {
    /// Schema version for upgrades
    pub version: AccountVersion,

    /// Emergency pause flag
    pub paused: bool,

    /// PDA bump seed
    pub bump: u8,

    /// Access manager transfer state for two-step propose/accept
    pub am_state: AccessManagerState,

    /// Reserved space for future fields
    pub _reserved: [u8; 256],
}

impl GMPAppState {
    pub const SEED: &'static [u8] = solana_ibc_types::GMPAppState::SEED;

    /// Get signer seeds for this app state
    /// Seeds: [`b"app_state"`, bump]
    pub fn signer_seeds(&self) -> Vec<Vec<u8>> {
        vec![Self::SEED.to_vec(), vec![self.bump]]
    }
}

/// Send call message (unvalidated input from user)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SendCallMsg {
    /// Source client identifier
    pub source_client: String,

    /// Caller-chosen packet sequence number
    pub sequence: u64,

    /// Timeout timestamp (unix seconds)
    pub timeout_timestamp: u64,

    /// Receiver address (string format to support any destination chain)
    pub receiver: String,

    /// Account salt
    pub salt: Vec<u8>,

    /// Call payload (instruction data + accounts)
    pub payload: Vec<u8>,

    /// Optional memo
    pub memo: String,

    /// Payload encoding format (e.g. `"application/x-protobuf"` or `"application/x-solidity-abi"`)
    pub encoding: String,
}

// Re-export types from proto crate
pub use crate::proto::{
    GmpAcknowledgement, GmpSolanaPayload, GmpValidationError, SolanaAccountMeta,
};

pub use solana_ibc_types::{CallResultStatus, GMPCallResult};

/// Persisted outcome of a cross-chain GMP call.
///
/// Created when the ICS26 router delivers an acknowledgement or timeout
/// callback to the GMP app. Stores either the IBC acknowledgement
/// commitment hash (on success/failure) or a timeout marker, so the
/// original sender can query the result on-chain after the round-trip
/// completes.
///
/// # PDA Seeds
/// `["gmp_result", source_client, sequence (little-endian u64)]`
#[account]
#[derive(InitSpace)]
pub struct GMPCallResultAccount {
    /// Account schema version for future upgrades.
    pub version: AccountVersion,
    /// Original sender pubkey.
    pub sender: Pubkey,
    /// Caller-chosen IBC packet sequence number.
    pub sequence: u64,
    /// Source client ID (light client on this chain tracking the destination).
    #[max_len(MAX_CLIENT_ID_LENGTH)]
    pub source_client: String,
    /// Destination client ID (light client on the destination chain).
    #[max_len(MAX_CLIENT_ID_LENGTH)]
    pub dest_client: String,
    /// Result status: acknowledgement (with IBC commitment) or timeout.
    pub status: CallResultStatus,
    /// Unix timestamp (seconds) when the result was recorded.
    pub result_timestamp: i64,
    /// PDA bump seed.
    pub bump: u8,
}

impl GMPCallResultAccount {
    /// Initialize the account with acknowledgement data.
    ///
    /// The acknowledgement commitment is computed using the IBC commitment format:
    /// `sha256(0x02 || sha256(ack))` where 0x02 is the IBC version byte.
    pub fn init_acknowledged(
        &mut self,
        msg: solana_ibc_types::OnAcknowledgementPacketMsg,
        sender: Pubkey,
        timestamp: i64,
        bump: u8,
    ) {
        self.version = AccountVersion::V1;
        self.sender = sender;
        self.sequence = msg.sequence;
        self.source_client = msg.source_client;
        self.dest_client = msg.dest_client;
        self.status = CallResultStatus::Acknowledgement(
            packet_acknowledgement_commitment_bytes32(std::slice::from_ref(&msg.acknowledgement))
                .expect("single ack is never empty"),
        );
        self.result_timestamp = timestamp;
        self.bump = bump;
    }

    /// Initialize the account with timeout data.
    pub fn init_timed_out(
        &mut self,
        msg: solana_ibc_types::OnTimeoutPacketMsg,
        sender: Pubkey,
        timestamp: i64,
        bump: u8,
    ) {
        self.version = AccountVersion::V1;
        self.sender = sender;
        self.sequence = msg.sequence;
        self.source_client = msg.source_client;
        self.dest_client = msg.dest_client;
        self.status = CallResultStatus::Timeout;
        self.result_timestamp = timestamp;
        self.bump = bump;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_ibc_types::Payload;

    fn test_payload() -> Payload {
        Payload {
            source_port: "gmpport".to_string(),
            dest_port: "gmpport".to_string(),
            version: "ics27-2".to_string(),
            encoding: "application/x-solidity-abi".to_string(),
            value: vec![],
        }
    }

    #[test]
    fn test_ack_commitment_computation() {
        let mut account = GMPCallResultAccount {
            version: AccountVersion::V1,
            sender: Pubkey::default(),
            sequence: 0,
            source_client: String::new(),
            dest_client: String::new(),
            status: CallResultStatus::Timeout, // Placeholder, overwritten by init_acknowledged
            result_timestamp: 0,
            bump: 0,
        };

        let ack_data = b"test acknowledgement data";
        let msg = solana_ibc_types::OnAcknowledgementPacketMsg {
            sequence: 42,
            source_client: "source-client".to_string(),
            dest_client: "dest-client".to_string(),
            payload: test_payload(),
            acknowledgement: ack_data.to_vec(),
            relayer: Pubkey::default(),
        };

        let sender = Pubkey::new_unique();
        account.init_acknowledged(msg, sender, 1_234_567_890, 255);

        let expected_commitment =
            packet_acknowledgement_commitment_bytes32(std::slice::from_ref(&ack_data.to_vec()))
                .unwrap();
        assert_eq!(
            account.status,
            CallResultStatus::Acknowledgement(expected_commitment)
        );
        assert_eq!(account.sequence, 42);
        assert_eq!(account.sender, sender);
    }

    #[test]
    fn test_timeout_status() {
        let mut account = GMPCallResultAccount {
            version: AccountVersion::V1,
            sender: Pubkey::default(),
            sequence: 0,
            source_client: String::new(),
            dest_client: String::new(),
            status: CallResultStatus::Timeout,
            result_timestamp: 0,
            bump: 0,
        };

        let msg = solana_ibc_types::OnTimeoutPacketMsg {
            sequence: 42,
            source_client: "source-client".to_string(),
            dest_client: "dest-client".to_string(),
            payload: test_payload(),
            relayer: Pubkey::default(),
        };

        let sender = Pubkey::new_unique();
        account.init_timed_out(msg, sender, 1_234_567_890, 255);

        assert_eq!(account.status, CallResultStatus::Timeout);
        assert_eq!(account.sequence, 42);
        assert_eq!(account.sender, sender);
    }
}
