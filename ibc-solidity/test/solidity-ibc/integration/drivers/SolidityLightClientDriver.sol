// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable immutable-vars-naming,no-empty-blocks

import { IICS02ClientMsgs } from "../../../../contracts/msgs/IICS02ClientMsgs.sol";

import { ILightClient } from "../../../../contracts/interfaces/ILightClient.sol";
import { ICS26Router } from "../../../../contracts/ICS26Router.sol";
import { IBCStoreUpgradeable } from "../../../../contracts/utils/IBCStoreUpgradeable.sol";
import { SolidityLightClient } from "../../utils/SolidityLightClient.sol";
import { ILightClientDriver } from "./ILightClientDriver.sol";
import { LightClientDriverTest } from "./LightClientDriverTest.sol";

/// @notice Drives a `SolidityLightClient`, which reads the counterparty router directly and needs no proofs.
contract SolidityLightClientDriver is ILightClientDriver {
    ILightClient public immutable lightClient;
    IBCStoreUpgradeable public immutable counterpartyRouter;

    constructor(ICS26Router counterparty) {
        lightClient = new SolidityLightClient(counterparty);
        counterpartyRouter = counterparty;
    }

    /// @inheritdoc ILightClientDriver
    function prove(bytes calldata)
        external
        pure
        returns (bytes memory, bytes memory, IICS02ClientMsgs.Height memory)
    { }
}

abstract contract WithSolidityLightClient is LightClientDriverTest {
    function _newDriver(ICS26Router counterparty) internal override returns (ILightClientDriver) {
        return new SolidityLightClientDriver(counterparty);
    }
}
