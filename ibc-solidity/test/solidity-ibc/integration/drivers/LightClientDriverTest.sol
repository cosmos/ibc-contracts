// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks

import { Test } from "forge-std/Test.sol";

import { ICS26Router } from "../../../../contracts/ICS26Router.sol";
import { IbcImpl } from "../../utils/IbcImpl.sol";
import { ILightClientDriver } from "./ILightClientDriver.sol";

/// @notice Base of integration suites that run on any light client and any chains; one mixin picks the client by
/// implementing `_newDriver`, another picks the chains by implementing `_beforeSetUp` and `_newIbcImpl`.
abstract contract LightClientDriverTest is Test {
    /// @notice Creates a driver whose light client is driven by `host` and tracks `counterparty`.
    function _newDriver(ICS26Router host, ICS26Router counterparty) internal virtual returns (ILightClientDriver);

    /// @notice Runs first in `setUp`, before the suite deploys anything, so a mixin can select a fork here.
    function _beforeSetUp() internal virtual;

    /// @notice Creates the IBC stack of the suite's chain at `slot`, where A is 0, B is 1 and C is 2.
    function _newIbcImpl(uint256 slot, address permit2) internal virtual returns (IbcImpl);
}

/// @notice Runs a suite on freshly deployed chains in the local test environment.
abstract contract WithFreshChains is LightClientDriverTest {
    function _beforeSetUp() internal override { }

    function _newIbcImpl(uint256, address permit2) internal override returns (IbcImpl) {
        return new IbcImpl(permit2);
    }
}
