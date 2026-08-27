// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length

import {Test} from "forge-std/Test.sol";

import {IICS02ClientMsgs} from "../../contracts/msgs/IICS02ClientMsgs.sol";
import {IICS26RouterMsgs} from "../../contracts/msgs/IICS26RouterMsgs.sol";
import {IIBCAppCallbacks} from "../../contracts/msgs/IIBCAppCallbacks.sol";

import {IICS26RouterErrors} from "../../contracts/errors/IICS26RouterErrors.sol";
import {IICS26Router} from "../../contracts/interfaces/IICS26Router.sol";
import {ILightClient} from "../../contracts/interfaces/ILightClient.sol";
import {IAccessManaged} from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import {IIBCApp} from "../../contracts/interfaces/IIBCApp.sol";

import {ICS26Router} from "../../contracts/ICS26Router.sol";
import {ICS20Lib} from "../../contracts/utils/ICS20Lib.sol";
import {ICS24Host} from "../../contracts/utils/ICS24Host.sol";
import {Strings} from "@openzeppelin-contracts/utils/Strings.sol";
import {ERC1967Proxy} from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {TestHelper} from "./utils/TestHelper.sol";
import {AccessManager} from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import {IBCRolesLib} from "../../contracts/utils/IBCRolesLib.sol";

contract ICS26RouterTest is Test {
    ICS26Router public ics26Router;

    TestHelper public testHelper = new TestHelper();

    address public relayer = makeAddr("relayer");
    address public idCustomizer = makeAddr("idCustomizer");
    address public mockClient = makeAddr("mockClient");

    function setUp() public {
        ICS26Router ics26RouterLogic = new ICS26Router();

        AccessManager accessManager = new AccessManager(address(this));

        ERC1967Proxy routerProxy = new ERC1967Proxy(
            address(ics26RouterLogic), abi.encodeCall(ICS26Router.initialize, (address(accessManager)))
        );

        ics26Router = ICS26Router(address(routerProxy));

        accessManager.setTargetFunctionRole(
            address(ics26Router), IBCRolesLib.ics26RelayerSelectors(), IBCRolesLib.RELAYER_ROLE
        );
        accessManager.setTargetFunctionRole(
            address(ics26Router), IBCRolesLib.ics26IdCustomizerSelectors(), IBCRolesLib.ID_CUSTOMIZER_ROLE
        );

        accessManager.grantRole(IBCRolesLib.RELAYER_ROLE, relayer, 0);
        accessManager.grantRole(IBCRolesLib.ID_CUSTOMIZER_ROLE, idCustomizer, 0);

        ics26Router.addClient(
            IICS02ClientMsgs.CounterpartyInfo("42-dummy-01", testHelper.COSMOS_MERKLE_PREFIX()), mockClient
        );
    }

    function test_success_addIBCAppUsingAddress() public {
        address mockApp = makeAddr("mockApp");
        string memory mockAppStr = Strings.toHexString(mockApp);

        vm.expectEmit();
        emit IICS26Router.IBCAppAdded(mockAppStr, mockApp);
        ics26Router.addIBCApp(mockApp);

        assertEq(mockApp, address(ics26Router.getIBCApp(mockAppStr)));
    }

    function test_success_addIBCAppUsingNamedPort() public {
        address mockApp = makeAddr("mockApp");

        vm.expectEmit();
        emit IICS26Router.IBCAppAdded(ICS20Lib.DEFAULT_PORT_ID, mockApp);
        vm.prank(idCustomizer);
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, mockApp);

        assertEq(mockApp, address(ics26Router.getIBCApp(ICS20Lib.DEFAULT_PORT_ID)));
    }

    function test_failure_addIBCAppUsingNamedPort() public {
        address mockApp = makeAddr("mockApp");
        // port is an address
        string memory mockAppStr = Strings.toHexString(mockApp);
        vm.prank(idCustomizer);
        vm.expectRevert(abi.encodeWithSelector(IICS26RouterErrors.IBCInvalidPortIdentifier.selector, mockAppStr));
        ics26Router.addIBCApp(mockAppStr, mockApp);

        // unauthorized
        address unauthorized = makeAddr("unauthorized");
        vm.prank(unauthorized);
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized));
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, mockApp);

        // reuse of the same port
        vm.prank(idCustomizer);
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, mockApp);
        vm.prank(idCustomizer);
        vm.expectRevert(
            abi.encodeWithSelector(IICS26RouterErrors.IBCPortAlreadyExists.selector, ICS20Lib.DEFAULT_PORT_ID)
        );
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, mockApp);
    }

    function test_unauthorizedSender() public {
        address mockApp = makeAddr("mockApp");
        vm.prank(idCustomizer);
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, mockApp);

        address unauthorizedSender = makeAddr("unauthorizedSender");

        IICS26RouterMsgs.MsgSendPacket memory msgSendPacket;
        msgSendPacket.payload.sourcePort = ICS20Lib.DEFAULT_PORT_ID;

        vm.prank(unauthorizedSender);
        vm.expectRevert(abi.encodeWithSelector(IICS26RouterErrors.IBCUnauthorizedSender.selector, unauthorizedSender));
        ics26Router.sendPacket(msgSendPacket);
    }

    function test_largeTimeout() public {
        address mockApp = makeAddr("mockApp");
        string memory mockPort = "mockport";

        vm.prank(idCustomizer);
        ics26Router.addIBCApp(mockPort, mockApp);

        uint64 timeoutTimestamp = uint64(block.timestamp + 2 days);
        string memory clientId = testHelper.FIRST_CLIENT_ID();

        vm.expectRevert(abi.encodeWithSelector(IICS26RouterErrors.IBCInvalidTimeoutDuration.selector, 1 days, 2 days));
        vm.prank(mockApp);
        ics26Router.sendPacket(
            IICS26RouterMsgs.MsgSendPacket({
                sourceClient: clientId,
                timeoutTimestamp: timeoutTimestamp,
                payload: IICS26RouterMsgs.Payload({
                    sourcePort: mockPort, destPort: mockPort, version: "", encoding: "", value: "0x"
                })
            })
        );
    }

    function test_RecvPacketWithFailedMembershipVerification() public {
        string memory counterpartyID = "42-dummy-01";
        bytes memory errorMsg = "Membership verification failed";

        vm.mockCallRevert(mockClient, ILightClient.verifyMembership.selector, errorMsg);

        address mockIcs20 = makeAddr("mockIcs20");
        vm.prank(idCustomizer);
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, address(mockIcs20));

        IICS26RouterMsgs.Payload[] memory payloads = new IICS26RouterMsgs.Payload[](1);
        payloads[0] = IICS26RouterMsgs.Payload({
            sourcePort: ICS20Lib.DEFAULT_PORT_ID,
            destPort: ICS20Lib.DEFAULT_PORT_ID,
            version: ICS20Lib.ICS20_VERSION,
            encoding: ICS20Lib.ICS20_ENCODING,
            value: "0x"
        });
        IICS26RouterMsgs.Packet memory packet = IICS26RouterMsgs.Packet({
            sequence: 1,
            sourceClient: counterpartyID,
            destClient: testHelper.FIRST_CLIENT_ID(),
            timeoutTimestamp: uint64(block.timestamp + 1000),
            payloads: payloads
        });

        IICS26RouterMsgs.MsgRecvPacket memory msgRecvPacket = IICS26RouterMsgs.MsgRecvPacket({
            packet: packet,
            proofCommitment: "0x", // doesn't matter
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: 0}) // doesn't matter
        });

        vm.expectRevert(errorMsg);
        vm.prank(relayer);
        ics26Router.recvPacket(msgRecvPacket);
    }

    function test_RecvPacketWithErrorAck() public {
        string memory counterpartyID = "42-dummy-01";

        vm.mockCall(mockClient, ILightClient.verifyMembership.selector, abi.encode(true));

        // We add an unusable ICS20Transfer app to the router
        address mockIcs20 = makeAddr("mockIcs20");
        vm.prank(idCustomizer);
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, mockIcs20);

        vm.mockCallRevert(mockIcs20, IIBCApp.onRecvPacket.selector, bytes("mockErr"));

        IICS26RouterMsgs.Payload[] memory payloads = new IICS26RouterMsgs.Payload[](1);
        payloads[0] = IICS26RouterMsgs.Payload({
            sourcePort: ICS20Lib.DEFAULT_PORT_ID,
            destPort: ICS20Lib.DEFAULT_PORT_ID,
            version: ICS20Lib.ICS20_VERSION,
            encoding: ICS20Lib.ICS20_ENCODING,
            value: "0x"
        });
        IICS26RouterMsgs.Packet memory packet = IICS26RouterMsgs.Packet({
            sequence: 1,
            sourceClient: counterpartyID,
            destClient: testHelper.FIRST_CLIENT_ID(),
            timeoutTimestamp: uint64(block.timestamp + 1000),
            payloads: payloads
        });

        IICS26RouterMsgs.MsgRecvPacket memory msgRecvPacket = IICS26RouterMsgs.MsgRecvPacket({
            packet: packet,
            proofCommitment: "0x", // doesn't matter
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: 0}) // doesn't matter
        });

        bytes[] memory expAcks = new bytes[](1);
        expAcks[0] = ICS24Host.UNIVERSAL_ERROR_ACK;

        vm.expectEmit();
        emit IICS26Router.WriteAcknowledgement(packet.destClient, packet.sequence, packet, expAcks);
        vm.prank(relayer);
        ics26Router.recvPacket(msgRecvPacket);
    }

    function test_RecvPacketWithOOG() public {
        string memory counterpartyID = "42-dummy-01";

        vm.mockCall(
            mockClient,
            ILightClient.verifyMembership.selector,
            abi.encode(true) // simulate successful membership verification
        );

        // We add a mock application that will run out of gas
        MockApplication mockApp = new MockApplication();
        vm.prank(idCustomizer);
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, address(mockApp));

        IICS26RouterMsgs.Payload[] memory payloads = new IICS26RouterMsgs.Payload[](1);
        payloads[0] = IICS26RouterMsgs.Payload({
            sourcePort: ICS20Lib.DEFAULT_PORT_ID,
            destPort: ICS20Lib.DEFAULT_PORT_ID,
            version: ICS20Lib.ICS20_VERSION,
            encoding: ICS20Lib.ICS20_ENCODING,
            value: "0x"
        });
        IICS26RouterMsgs.Packet memory packet = IICS26RouterMsgs.Packet({
            sequence: 1,
            sourceClient: counterpartyID,
            destClient: testHelper.FIRST_CLIENT_ID(),
            timeoutTimestamp: uint64(block.timestamp + 1000),
            payloads: payloads
        });

        IICS26RouterMsgs.MsgRecvPacket memory msgRecvPacket = IICS26RouterMsgs.MsgRecvPacket({
            packet: packet,
            proofCommitment: "0x", // doesn't matter
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: 0}) // doesn't matter
        });

        vm.expectRevert(abi.encodeWithSelector(IICS26RouterErrors.IBCFailedCallback.selector));
        vm.prank(relayer);
        ics26Router.recvPacket{gas: 900_000}(msgRecvPacket);
    }
}

contract MockApplication is Test {
    function onRecvPacket(IIBCAppCallbacks.OnRecvPacketCallback calldata) external pure returns (bytes memory) {
        for (uint256 i = 0; i < 14_000; ++i) {
            uint256 x;
            x = x * i;
        }

        return bytes("mock");
    }
}
