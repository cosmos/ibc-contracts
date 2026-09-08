// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title SP1 Messages
/// @notice This interface defines the structure of messages used in the SP1 program.
interface ISP1Msgs {
    /// @notice The SP1 proof that can be submitted to the SP1Verifier contract.
    /// @dev vKey must be verified before sending this to the SP1 program.
    /// @param vKey The verification key for the program.
    /// @param publicValues The public values for the program.
    /// @param proof The proof for the program.
    struct SP1Proof {
        bytes32 vKey;
        bytes publicValues;
        bytes proof;
    }
}
