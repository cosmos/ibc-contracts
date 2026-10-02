// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";

import { QBFTSimSuite } from "./utils/QBFTSimSuite.sol";
import { SimHeader } from "./utils/SimHeader.sol";

/// @dev Drives an honest simulated chain with validator churn and relays random headers against random trusted
/// heights, in any order.
contract QBFTSimHandler is Test {
    QBFTSimSuite public immutable SIM;
    IBesuLightClient public immutable CLIENT;

    uint64[] public trustedHeights;
    uint64 public maxHeight;

    constructor(QBFTSimSuite sim, IBesuLightClient client) {
        SIM = sim;
        CLIENT = client;
        maxHeight = sim.tipHeight();
        trustedHeights.push(maxHeight);
    }

    function produceBlocks(uint256 count) external {
        SIM.produceBlocks(bound(count, 1, 3));
    }

    function rotateValidator(uint256 seed) external {
        address[] memory validators = SIM.validators();
        SIM.removeValidator(validators[seed % validators.length]);
        SIM.addValidator();
    }

    function updateClient(uint256 trustedSeed, uint256 heightSeed) external {
        uint64 trustedHeight = trustedHeights[trustedSeed % trustedHeights.length];
        uint64 tip = SIM.tipHeight();
        if (tip <= trustedHeight) {
            return;
        }
        uint64 height = uint64(bound(heightSeed, trustedHeight + 1, tip));
        bytes memory update = SIM.updateMsg(trustedHeight, height);

        if (CLIENT.updateClient(update) == ILightClientMsgs.UpdateResult.Update) {
            trustedHeights.push(height);
            if (height > maxHeight) {
                maxHeight = height;
            }
        }
    }

    function trustedHeightCount() external view returns (uint256) {
        return trustedHeights.length;
    }
}

/// @dev Runs once per consensus mode through `QBFTSimInvariantQBFTTest` and `QBFTSimInvariantIBFT2Test`.
abstract contract QBFTSimInvariantTest is Test {
    QBFTSimSuite internal sim;
    IBesuLightClient internal client;
    QBFTSimHandler internal handler;

    function _mode() internal pure virtual returns (SimHeader.Mode);

    function setUp() public {
        sim = new QBFTSimSuite(_mode());
        sim.addValidators(4);
        sim.produceBlock();
        client = sim.deployLightClient(1 days, 10, IBesuLightClientMsgs.TrustThreshold(2, 3));
        handler = new QBFTSimHandler(sim, client);
        targetContract(address(handler));
    }

    /// @dev An honest chain never freezes the client, `latestHeight` tracks the highest relayed header even when
    /// older ones are filled in afterwards, and every stored consensus state is the one the chain produced.
    function invariant_honestRelay() public view {
        IBesuLightClientMsgs.ClientState memory state =
            abi.decode(client.getClientState(), (IBesuLightClientMsgs.ClientState));
        assertFalse(state.isFrozen);
        assertEq(state.latestHeight.revisionHeight, handler.maxHeight());

        for (uint256 i = 0; i < handler.trustedHeightCount(); ++i) {
            uint64 height = handler.trustedHeights(i);
            assertEq(client.getConsensusStateHash(height), keccak256(abi.encode(sim.consensusState(height))));
        }
    }
}

/// forge-config: default.invariant.runs = 64
/// forge-config: default.invariant.depth = 64
contract QBFTSimInvariantQBFTTest is QBFTSimInvariantTest {
    function _mode() internal pure override returns (SimHeader.Mode) {
        return SimHeader.Mode.QBFT;
    }
}

/// forge-config: default.invariant.runs = 64
/// forge-config: default.invariant.depth = 64
contract QBFTSimInvariantIBFT2Test is QBFTSimInvariantTest {
    function _mode() internal pure override returns (SimHeader.Mode) {
        return SimHeader.Mode.IBFT2;
    }
}
