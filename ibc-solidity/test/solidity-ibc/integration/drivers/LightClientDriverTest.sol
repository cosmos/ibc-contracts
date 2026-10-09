// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { ICS26Router } from "../../../../contracts/ICS26Router.sol";
import { ILightClientDriver } from "./ILightClientDriver.sol";

/// @notice Base of integration suites that run on any light client; a mixin picks the client by implementing
/// `_newDriver`.
abstract contract LightClientDriverTest is Test {
    /// @notice Creates a driver whose light client tracks `counterparty`.
    function _newDriver(ICS26Router counterparty) internal virtual returns (ILightClientDriver);
}
