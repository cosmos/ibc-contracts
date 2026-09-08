// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title Client Messages
/// @notice Interface defining ICS02 Client Messages
interface IICS02ClientMsgs {
    /// @notice Counterparty client information.
    /// @dev merklePrefix is structured as a list of bytes representing the keys in nested merkle trees.
    /// @custom:spec
    /// https://github.com/cosmos/ibc/blob/67fe813f7e4ec603a7c5dec35bc654f3b012afda/spec/micro/README.md?plain=1#L91
    /// @param clientId The client identifier from the counterparty chain.
    /// @param merklePrefix The counterparty chain's merkle prefix.
    struct CounterpartyInfo {
        string clientId;
        bytes[] merklePrefix;
    }

    /// @notice Height of the counterparty chain
    /// @param revisionNumber The revision number of the counterparty chain
    /// @param revisionHeight The height of the counterparty chain
    struct Height {
        uint64 revisionNumber;
        uint64 revisionHeight;
    }
}
