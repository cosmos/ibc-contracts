// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIFTSendCallConstructor } from "../interfaces/IIFTSendCallConstructor.sol";

/// @title IFT Messages
/// @notice Interface defining IFT data structures
interface IIFTMsgs {
    /// @notice IFTBridge represents a counterparty IFT contract on a different chain connected via an IBC Light Client
    /// @param clientId The IBC client identifier on the local chain representing the counterparty chain
    /// @param counterpartyIFTAddress The address of the IFT contract on the counterparty chain
    /// @param iftSendCallConstructor The constructor used to build mint call data for the counterparty chain
    struct IFTBridge {
        string clientId;
        string counterpartyIFTAddress;
        IIFTSendCallConstructor iftSendCallConstructor;
    }

    /// @notice PendingTransfer represents a transfer that has been initiated but not yet completed
    /// @dev Used to refund tokens in case of transfer failure
    /// @param sender The address of the sender who initiated the transfer
    /// @param amount The amount of tokens involved in the pending transfer
    struct PendingTransfer {
        address sender;
        uint256 amount;
    }

    /// @notice Direction of a rate limited IFT flow
    /// @dev Inbound covers mints received from a counterparty chain, Outbound covers burns initiated by local
    /// transfers to a counterparty chain
    enum IFTRateLimitDirection {
        Inbound,
        Outbound
    }
}
