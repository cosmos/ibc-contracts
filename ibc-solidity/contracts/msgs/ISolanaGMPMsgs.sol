// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title Solana GMP Messages
/// @notice Interface defining Solana-specific GMP message types
interface ISolanaGMPMsgs {
    /// @notice GMPSolanaPayload is the ABI-encoded payload for Solana GMP execution.
    /// @param packedAccounts Packed account entries (34 bytes each: pubkey + is_signer + is_writable)
    /// @param instructionData Raw instruction data for the target program
    /// @param prefundLamports Lamports to prefund the caller PDA
    struct GMPSolanaPayload {
        bytes packedAccounts;
        bytes instructionData;
        uint64 prefundLamports;
    }
}
