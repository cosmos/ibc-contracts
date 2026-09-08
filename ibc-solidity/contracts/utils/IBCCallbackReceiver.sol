// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIBCSenderCallbacks } from "../interfaces/IIBCSenderCallbacks.sol";

import { ERC165 } from "@openzeppelin-contracts/utils/introspection/ERC165.sol";

/// @title IBC Callback Receiver
/// @notice An abstract contract that implements the ERC165 interface for IIBCSenderCallbacks.
abstract contract IBCCallbackReceiver is IIBCSenderCallbacks, ERC165 {
    /// @inheritdoc ERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override(ERC165) returns (bool) {
        return interfaceId == type(IIBCSenderCallbacks).interfaceId || super.supportsInterface(interfaceId);
    }
}
