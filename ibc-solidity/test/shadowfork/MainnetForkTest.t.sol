// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings

import { Test } from "forge-std/Test.sol";

import { ICS20Transfer } from "../../contracts/ICS20Transfer.sol";
import { ICS26Router } from "../../contracts/ICS26Router.sol";
import { ICS20Lib } from "../../contracts/utils/ICS20Lib.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import { IICS02ClientAccessControlled } from "../../contracts/interfaces/IICS02Client.sol";
import { IICS26RouterAccessControlled } from "../../contracts/interfaces/IICS26Router.sol";

import { DeployAccessManagerWithRoles } from "../../scripts/deployments/DeployAccessManagerWithRoles.sol";

contract MainnetForkTest is Test, DeployAccessManagerWithRoles {
    // solhint-disable-next-line var-name-mixedcase
    string public ETH_RPC_URL = vm.envString("ETH_RPC_URL");

    string public clientId = "cosmoshub-0";

    // WARNING: As the mainnet contracts may not be up to date, some interface functions may not work as expected
    // In this case, you should add the necessary interfaces to the `./utils/vX.Y.Z/` directory

    ICS26Router public ics26Proxy = ICS26Router(0x3aF134307D5Ee90faa2ba9Cdba14ba66414CF1A7);
    ICS20Transfer public ics20Proxy = ICS20Transfer(0xa348CfE719B63151F228e3C30EB424BA5a983012);
    AccessManager public accessManager = AccessManager(0x3fa3f45acE1645614c80679AeEcE0A82A93c77Ec);
    address public timelockedAdmin = 0xb3999B2D30dD8c9faEcE5A8a503fAe42b8b1b614;
    address public relayer = 0xC4C09A23dDBd1fF0f313885265113F83622284C2;

    function setUp() public {
        uint256 forkId = vm.createFork(ETH_RPC_URL);
        vm.selectFork(forkId);
    }

    function test_validate_constants() public view {
        address transfer = address(ics26Proxy.getIBCApp(ICS20Lib.DEFAULT_PORT_ID));
        assertEq(transfer, address(ics20Proxy), "ICS20Proxy address is not correct");

        address router = ics20Proxy.ics26();
        assertEq(router, address(ics26Proxy), "ICS26Router address is not correct");

        address ics07 = address(ics26Proxy.getClient(clientId));
        assertTrue(ics07 != address(0), "Client not found");
    }

    function test_upgrade_ics26PublicSelectors() public {
        address user = makeAddr("user");
        bytes[] memory merklePrefix = new bytes[](2);
        merklePrefix[0] = bytes("ibc");
        IICS02ClientMsgs.CounterpartyInfo memory counterpartyInfo =
            IICS02ClientMsgs.CounterpartyInfo("42-dummy-01", merklePrefix);

        address newLogic = address(new ICS26Router());
        vm.prank(timelockedAdmin);
        ics26Proxy.upgradeToAndCall(newLogic, "");

        // the newly restricted selectors default to ADMIN_ROLE after the upgrade
        vm.prank(user);
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, user));
        ics26Proxy.addIBCApp(makeAddr("mockApp"));
        vm.prank(user);
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, user));
        ics26Proxy.addClient(counterpartyInfo, makeAddr("mockClient"));

        vm.prank(timelockedAdmin);
        accessManager.setTargetFunctionRole(
            address(ics26Proxy), IBCRolesLib.ics26PublicSelectors(), IBCRolesLib.PUBLIC_ROLE
        );

        vm.prank(user);
        ics26Proxy.addIBCApp(makeAddr("mockApp"));
        vm.prank(user);
        ics26Proxy.addClient(counterpartyInfo, makeAddr("mockClient"));
    }

    function test_upgrade_ics26VerifySelectors() public {
        bytes4[] memory verifySelectors = new bytes4[](2);
        verifySelectors[0] = IICS02ClientAccessControlled.verifyMembership.selector;
        verifySelectors[1] = IICS02ClientAccessControlled.verifyNonMembership.selector;
        uint64 relayerRole =
            accessManager.getTargetFunctionRole(address(ics26Proxy), IICS26RouterAccessControlled.recvPacket.selector);

        address newLogic = address(new ICS26Router());
        vm.prank(timelockedAdmin);
        ics26Proxy.upgradeToAndCall(newLogic, "");

        // the new verification selectors default to ADMIN_ROLE after the upgrade
        for (uint256 i = 0; i < verifySelectors.length; ++i) {
            assertEq(
                accessManager.getTargetFunctionRole(address(ics26Proxy), verifySelectors[i]), IBCRolesLib.ADMIN_ROLE
            );
        }

        // the upgrade proposal maps them to the role that relays packets
        vm.prank(timelockedAdmin);
        accessManager.setTargetFunctionRole(address(ics26Proxy), verifySelectors, relayerRole);
        for (uint256 i = 0; i < verifySelectors.length; ++i) {
            assertEq(accessManager.getTargetFunctionRole(address(ics26Proxy), verifySelectors[i]), relayerRole);
        }
    }
}
