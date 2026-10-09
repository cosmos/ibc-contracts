// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-indexed-events

import { IFTOwnable } from "../../../contracts/utils/IFTOwnable.sol";

/// @title IFTHooksMock
/// @notice IFT that emits the context of every bridge hook and can be toggled to reject them
contract IFTHooksMock is IFTOwnable {
    bool public rejectHooks;

    event BeforeIFTTransfer(address sender, string clientId, string receiver, uint256 amount, uint64 timeoutTimestamp);
    event BeforeIFTMint(string clientId, address receiver, uint256 amount);
    event BeforeIFTRefund(string clientId, uint64 sequence, address sender, uint256 amount);

    error HookRejected();

    function setRejectHooks(bool reject) external {
        rejectHooks = reject;
    }

    function _beforeIFTTransfer(
        address sender,
        string calldata clientId,
        string calldata receiver,
        uint256 amount,
        uint64 timeoutTimestamp
    )
        internal
        override
    {
        require(!rejectHooks, HookRejected());
        emit BeforeIFTTransfer(sender, clientId, receiver, amount, timeoutTimestamp);
    }

    function _beforeIFTMint(string memory clientId, address receiver, uint256 amount) internal override {
        require(!rejectHooks, HookRejected());
        emit BeforeIFTMint(clientId, receiver, amount);
    }

    function _beforeIFTRefund(
        string memory clientId,
        uint64 sequence,
        address sender,
        uint256 amount
    )
        internal
        override
    {
        require(!rejectHooks, HookRejected());
        emit BeforeIFTRefund(clientId, sequence, sender, amount);
    }
}
