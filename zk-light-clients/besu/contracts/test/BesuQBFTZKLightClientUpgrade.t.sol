// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Test } from "forge-std/Test.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import { ERC1967Utils } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Utils.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";

import { BesuQBFTZKLightClient } from "../src/BesuQBFTZKLightClient.sol";
import { BesuZKTypes } from "../src/BesuZKTypes.sol";
import { MockSP1Verifier } from "./mocks/MockSP1Verifier.sol";
import { LightClientProxy } from "./utils/LightClientProxy.sol";

contract BesuQBFTZKLightClientUpgradeTest is Test {
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.lightclient.storage.BesuQBFTZKLightClient"))
    /// - 1)) & ~bytes32(uint256(0xff)), computed here rather than copied from the contract.
    bytes32 private immutable STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("ibc.lightclient.storage.BesuQBFTZKLightClient")) - 1)
    ) & ~bytes32(uint256(0xff));

    address private constant SOURCE_ROUTER = address(0xBEEF);
    address private constant STRANGER = address(0x5157);
    bytes private constant PROOF = hex"01020304";
    uint64 private constant UPGRADER = 7;

    MockSP1Verifier private verifier;
    AccessManager private manager;
    BesuQBFTZKLightClient private client;

    function setUp() public {
        verifier = new MockSP1Verifier();
        manager = new AccessManager(address(this));
        client = LightClientProxy.deploy(address(manager), _params());
        vm.warp(1_100);
    }

    function test_anAdminUpgradesWithoutAnyRoleSetUp() public {
        address next = address(new BesuQBFTZKLightClient());
        client.upgradeToAndCall(next, "");
        assertEq(_implementation(), next);
    }

    function test_upgradeKeepsEveryField() public {
        BesuZKTypes.UpdatePublicValues memory values = BesuZKTypes.UpdatePublicValues({
            schemaVersion: 1,
            trustedHeight: 7,
            trustedValidatorSetHash: bytes32(uint256(0x2222)),
            verifiedHeight: 9,
            stateRoot: bytes32(uint256(0x3333)),
            blockTimestamp: 1_050,
            newValidatorSetHash: bytes32(uint256(0x4444))
        });
        bytes memory publicValues = abi.encode(values);
        verifier.setExpected(bytes32(uint256(1)), publicValues, PROOF);
        client.updateClient(abi.encode(BesuZKTypes.SP1ProofEnvelope(publicValues, PROOF)));
        bytes memory clientState = client.getClientState();

        client.upgradeToAndCall(address(new BesuQBFTZKLightClient()), "");

        assertEq(client.getClientState(), clientState);
        assertEq(client.latestHeight(), 9);
        (bool found, BesuZKTypes.ConsensusState memory state) = client.getConsensusState(9);
        assertTrue(found);
        assertEq(state.stateRoot, values.stateRoot);
        (found, state) = client.getConsensusState(7);
        assertTrue(found);
        assertEq(state.stateRoot, bytes32(uint256(0x1111)));
        assertEq(client.LOCAL_IBC_ROUTER(), address(this));
        assertEq(client.SOURCE_IBC_ROUTER(), SOURCE_ROUTER);
        assertEq(client.SP1_VERIFIER(), address(verifier));
        assertEq(client.UPDATE_CLIENT_VKEY(), bytes32(uint256(1)));
        assertEq(client.MEMBERSHIP_VKEY(), bytes32(uint256(2)));
        assertEq(client.NON_MEMBERSHIP_VKEY(), bytes32(uint256(3)));
        assertEq(client.authority(), address(manager));
    }

    function test_refusesAnUpgradeFromAnyoneTheAuthorityDoesNotAllow() public {
        address next = address(new BesuQBFTZKLightClient());
        vm.prank(STRANGER);
        vm.expectRevert(
            abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, STRANGER)
        );
        client.upgradeToAndCall(next, "");
    }

    /// @dev The check is the authority's `canCall` for `upgradeToAndCall` on the proxy, so a role
    /// mapped to that selector is what grants it.
    function test_theAuthorityDecidesWhoUpgrades() public {
        bytes4[] memory selectors = new bytes4[](1);
        selectors[0] = UUPSUpgradeable.upgradeToAndCall.selector;
        manager.setTargetFunctionRole(address(client), selectors, UPGRADER);
        manager.grantRole(UPGRADER, STRANGER, 0);

        address next = address(new BesuQBFTZKLightClient());
        vm.prank(STRANGER);
        client.upgradeToAndCall(next, "");
        assertEq(_implementation(), next);
    }

    function test_anImplementationCannotBeUpgradedDirectly() public {
        BesuQBFTZKLightClient implementation = BesuQBFTZKLightClient(_implementation());
        address next = address(new BesuQBFTZKLightClient());
        vm.expectRevert(UUPSUpgradeable.UUPSUnauthorizedCallContext.selector);
        implementation.upgradeToAndCall(next, "");
    }

    function test_stateLivesInTheNamespacedSlot() public view {
        assertEq(vm.load(address(client), STORAGE_SLOT), bytes32(uint256(uint160(address(this)))));
        assertEq(
            vm.load(address(client), bytes32(uint256(STORAGE_SLOT) + 1)),
            bytes32(uint256(uint160(SOURCE_ROUTER)))
        );
        // clientState, packed after the six fields: latestHeight | trustingPeriod | drift.
        assertEq(
            vm.load(address(client), bytes32(uint256(STORAGE_SLOT) + 6)),
            bytes32((uint256(20) << 128) | (uint256(1_000) << 64) | uint256(7))
        );
        assertEq(vm.load(address(client), bytes32(0)), bytes32(0));
        assertEq(vm.load(address(client), bytes32(uint256(1))), bytes32(0));
    }

    function _implementation() private view returns (address) {
        return address(uint160(uint256(vm.load(address(client), ERC1967Utils.IMPLEMENTATION_SLOT))));
    }

    function _params() private view returns (BesuQBFTZKLightClient.InitParams memory) {
        return BesuQBFTZKLightClient.InitParams({
            localIbcRouter: address(this),
            sourceIbcRouter: SOURCE_ROUTER,
            sp1Verifier: address(verifier),
            updateClientVKey: bytes32(uint256(1)),
            membershipVKey: bytes32(uint256(2)),
            nonMembershipVKey: bytes32(uint256(3)),
            initialHeight: 7,
            initialStateRoot: bytes32(uint256(0x1111)),
            initialBlockTimestamp: 1_000,
            initialValidatorSetHash: bytes32(uint256(0x2222)),
            trustingPeriodSeconds: 1_000,
            maxClockDriftSeconds: 20
        });
    }
}
