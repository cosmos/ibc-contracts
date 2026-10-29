// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings

import { ICS20Transfer } from "../../contracts/ICS20Transfer.sol";
import { ICS26Router } from "../../contracts/ICS26Router.sol";
import { ICS27GMP } from "../../contracts/ICS27GMP.sol";
import { ICS27Account } from "../../contracts/utils/ICS27Account.sol";
import { ICS20Lib } from "../../contracts/utils/ICS20Lib.sol";
import { ICS27Lib } from "../../contracts/utils/ICS27Lib.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import { BeaconProxy } from "@openzeppelin-contracts/proxy/beacon/BeaconProxy.sol";
import { UpgradeableBeacon } from "@openzeppelin-contracts/proxy/beacon/UpgradeableBeacon.sol";
import { Errors } from "@openzeppelin-contracts/utils/Errors.sol";
import { IICS02ClientAccessControlled } from "../../contracts/interfaces/IICS02Client.sol";
import { IICS26RouterAccessControlled } from "../../contracts/interfaces/IICS26Router.sol";

import { DeployAccessManagerWithRoles } from "../../scripts/deployments/DeployAccessManagerWithRoles.sol";
import { MulticallTargetMock } from "../solidity-ibc/mocks/MulticallTargetMock.sol";
import { MainnetDeployment } from "./utils/MainnetIbcImpl.sol";

contract MainnetForkTest is MainnetDeployment, DeployAccessManagerWithRoles {
    // solhint-disable-next-line var-name-mixedcase
    string public ETH_RPC_URL = vm.envString("ETH_RPC_URL");

    string public clientId = "cosmoshub-0";

    // WARNING: As the mainnet contracts may not be up to date, some interface functions may not work as expected
    // In this case, you should add the necessary interfaces to the `./utils/vX.Y.Z/` directory

    ICS26Router public ics26Proxy = ICS26Router(ICS26_ROUTER);
    ICS20Transfer public ics20Proxy = ICS20Transfer(ICS20_TRANSFER);
    ICS27GMP public ics27Proxy = ICS27GMP(ICS27_GMP);
    AccessManager public accessManager = AccessManager(ACCESS_MANAGER);
    address public timelockedAdmin = TIMELOCKED_ADMIN;
    address public relayer = RELAYER;

    function setUp() public {
        _selectMainnetFork(ETH_RPC_URL);
    }

    function test_validate_constants() public view {
        address transfer = address(ics26Proxy.getIBCApp(ICS20Lib.DEFAULT_PORT_ID));
        assertEq(transfer, address(ics20Proxy), "ICS20Proxy address is not correct");

        address router = ics20Proxy.ics26();
        assertEq(router, address(ics26Proxy), "ICS26Router address is not correct");

        address gmp = address(ics26Proxy.getIBCApp(ICS27Lib.DEFAULT_PORT_ID));
        assertEq(gmp, address(ics27Proxy), "ICS27Proxy address is not correct");
        assertEq(ics27Proxy.ics26(), address(ics26Proxy), "ICS26Router address is not correct");

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

    function test_upgrade_ics27Account() public {
        UpgradeableBeacon beacon = UpgradeableBeacon(ics27Proxy.getAccountBeacon());
        // An account like the ones ICS27GMP deploys, which all share the beacon
        ICS27Account account = ICS27Account(
            address(new BeaconProxy(address(beacon), abi.encodeCall(ICS27Account.initialize, (address(ics27Proxy)))))
        );
        address target = address(new MulticallTargetMock(address(0)));
        bytes memory data = abi.encodeCall(MulticallTargetMock.burnGas, ());

        // the deployed account reports running out of gas as an error, which the router acknowledges
        vm.expectRevert(Errors.FailedCall.selector);
        vm.prank(address(ics27Proxy));
        account.functionCall{ gas: 1_000_000 }(target, data);

        address newLogic = address(new ICS27Account());
        vm.prank(timelockedAdmin);
        ics27Proxy.upgradeAccountTo(newLogic);
        assertEq(beacon.implementation(), newLogic, "Account beacon is not upgraded");
        assertEq(account.ics27(), address(ics27Proxy), "Account storage is not preserved");

        // the upgraded account reverts without data, so the router fails the relay
        vm.expectRevert(bytes(""));
        vm.prank(address(ics27Proxy));
        account.functionCall{ gas: 1_000_000 }(target, data);
    }
}
