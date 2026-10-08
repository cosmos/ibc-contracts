// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable immutable-vars-naming

import { IICS02ClientMsgs } from "../../../contracts/msgs/IICS02ClientMsgs.sol";
import { ILightClientMsgs } from "../../../contracts/msgs/ILightClientMsgs.sol";
import { IBesuLightClientMsgs } from "../../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";

import { ILightClient } from "../../../contracts/interfaces/ILightClient.sol";
import { ICS26Router } from "../../../contracts/ICS26Router.sol";
import { IBCStoreUpgradeable } from "../../../contracts/utils/IBCStoreUpgradeable.sol";
import { Math } from "@openzeppelin-contracts/utils/math/Math.sol";
import { ILightClientDriver } from "../../solidity-ibc/integration/drivers/ILightClientDriver.sol";
import { LightClientDriverTest } from "../../solidity-ibc/integration/drivers/LightClientDriverTest.sol";
import { QBFTSimSuite } from "./QBFTSimSuite.sol";
import { SimHeader } from "./SimHeader.sol";

/// @notice Drives a Besu light client that tracks a `QBFTSimSuite` chain mirroring the counterparty router.
/// @dev Each proof mirrors the proven commitment into the sim, produces a block and updates the client to it. Blocks
/// are stamped with the EVM clock, so the proven counterparty time is the current time as timeouts expect.
contract BesuSimDriver is ILightClientDriver {
    QBFTSimSuite public immutable sim;
    ILightClient public immutable lightClient;
    IBCStoreUpgradeable private immutable _counterpartyRouter;

    uint64 private _trustedHeight;

    constructor(SimHeader.Mode mode, ICS26Router counterparty) {
        sim = new QBFTSimSuite(mode);
        sim.addValidators(4);
        _trustedHeight = _produceBlock();
        lightClient = sim.deployLightClient(1 days, 10, IBesuLightClientMsgs.TrustThreshold(2, 3));
        _counterpartyRouter = counterparty;
    }

    /// @inheritdoc ILightClientDriver
    function prove(bytes calldata path)
        external
        returns (bytes memory updateMsg, bytes memory proof, IICS02ClientMsgs.Height memory proofHeight)
    {
        sim.syncCommitment(_counterpartyRouter, path);
        uint64 height = _produceBlock();
        updateMsg = sim.updateMsg(_trustedHeight, height);
        _trustedHeight = height;

        if (_counterpartyRouter.getCommitment(keccak256(path)) != 0) {
            ILightClientMsgs.MsgVerifyMembership memory msg_ = sim.membershipMsg(height, path);
            return (updateMsg, msg_.proof, msg_.proofHeight);
        }
        ILightClientMsgs.MsgVerifyNonMembership memory nonMsg = sim.nonMembershipMsg(height, path);
        return (updateMsg, nonMsg.proof, nonMsg.proofHeight);
    }

    /// @dev Stamps the block with the current time, or one second after the tip if that is later, so the sim only
    /// warps the clock when several blocks land in the same second.
    function _produceBlock() private returns (uint64) {
        SimHeader.Data memory h = sim.nextBlock();
        h.timestamp = uint64(Math.max(sim.blockAt(sim.tipHeight()).timestamp + 1, block.timestamp));
        sim.commit(sim.seal(h));
        return h.number;
    }
}

abstract contract WithBesuQBFTSim is LightClientDriverTest {
    function _newDriver(ICS26Router counterparty) internal override returns (ILightClientDriver) {
        return new BesuSimDriver(SimHeader.Mode.QBFT, counterparty);
    }
}

abstract contract WithBesuIBFT2Sim is LightClientDriverTest {
    function _newDriver(ICS26Router counterparty) internal override returns (ILightClientDriver) {
        return new BesuSimDriver(SimHeader.Mode.IBFT2, counterparty);
    }
}
